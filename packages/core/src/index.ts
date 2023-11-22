import * as Schema from "@effect/schema/Schema";
import { Client, components, paths } from "@pointguard/api-client";

export * from "@pointguard/api-client";

export const Retriable = Symbol.for("@pointguard/core/Retriable");
const isNonRetriable = (e: unknown): boolean =>
  typeof e === "object" &&
  e !== null &&
  (("retry" in e && e.retry === false) ||
    (Retriable in e && e[Retriable] === false));
export const isRetriable = (e: unknown): boolean => !isNonRetriable(e);

export class RejectedError extends Error {
  [Retriable] = false;
  name = "RejectedJobError";
}

type AllEnqueueParameters =
  paths["/api/v1/tasks"]["post"]["requestBody"]["content"]["application/json"];

const EnqueueOptions = Schema.struct({
  runAt: Schema.string.pipe(
    Schema.dateFromString,
    Schema.nullable,
    Schema.optional
  ),
  maxRetries: Schema.number.pipe(
    Schema.greaterThanOrEqualTo(0),
    Schema.nullable,
    Schema.optional
  ),
  name: Schema.string.pipe(Schema.nullable, Schema.optional),
}) satisfies Schema.Schema<
  Omit<AllEnqueueParameters, "data" | "jobName" | "endpoint">,
  any
>;

const encodeEnqueueOptions = Schema.encodeSync(EnqueueOptions);

type EnqueueOptionsFields = keyof Schema.Schema.To<typeof EnqueueOptions>;
type ChainedEnqueuer<Input> = {
  [K in EnqueueOptionsFields as `with${Capitalize<K>}`]: (
    value:
      | Schema.Schema.To<typeof EnqueueOptions>[K]
      | (() => Schema.Schema.To<typeof EnqueueOptions>[K])
  ) => ChainedEnqueuer<Input>;
} & {
  enqueue(
    input: Input,
    opts?: Partial<Schema.Schema.To<typeof EnqueueOptions>>
  ): Promise<void>;
};

type LazyEnqueueOptions = {
  [K in EnqueueOptionsFields]:
    | Schema.Schema.To<typeof EnqueueOptions>[K]
    | (() => Schema.Schema.To<typeof EnqueueOptions>[K]);
};

function createChainedEnqueuer<Input>(opts: {
  jobName: string;
  pointguardBaseUrl: URL | string | undefined;
  jobHandlerUrl: URL | string | undefined;
  opts: Partial<LazyEnqueueOptions>;
}): ChainedEnqueuer<Input> {
  const setOption = <K extends EnqueueOptionsFields>(
    key: K,
    value:
      | Schema.Schema.To<typeof EnqueueOptions>[K]
      | (() => Schema.Schema.To<typeof EnqueueOptions>[K])
  ) =>
    createChainedEnqueuer({
      ...opts,
      opts: {
        ...opts.opts,
        [key]: value,
      },
    });

  return {
    withMaxRetries: (value) => setOption("maxRetries", value),
    withName: (value) => setOption("name", value),
    withRunAt: (value) => setOption("runAt", value),
    enqueue: async (input, overrides) =>
      enqueueJob({
        client: new Client({
          baseUrl: String(
            opts.pointguardBaseUrl || getDefaultPointgaurgBaseUrl()
          ),
        }),
        jobHandlerUrl: opts.jobHandlerUrl,
        jobName: opts.jobName,
        input: input,
        opts: {
          ...Object.fromEntries(
            Object.entries(opts.opts).map(([key, maybeFn]) => {
              const value = typeof maybeFn === "function" ? maybeFn() : maybeFn;
              return [key, value] as const;
            }, {})
          ),
          ...overrides,
        },
      }),
  };
}

async function enqueueJob({
  jobName,
  opts,
  input,
  client,
  jobHandlerUrl,
}: {
  jobName: string;
  opts?: Schema.Schema.To<typeof EnqueueOptions>;
  input: unknown;
  jobHandlerUrl: URL | string | undefined;
  client: Client;
}) {
  const jobOptions = encodeEnqueueOptions(opts ?? {});
  const response = await fetch(
    client.request("/api/v1/tasks", "post", {
      data: input,
      jobName,
      endpoint: String(jobHandlerUrl || getDefaultJobHandlerUrl()),
      ...jobOptions,
    }),
    {
      cache: "no-cache",
    }
  );
  if (!response.ok) {
    throw new Error(`failed to enqueue job ${jobName}: ${response.statusText}`);
  }
}

export interface Job<Input> extends ChainedEnqueuer<Input> {
  name: string;

  handler(
    input: Input,
    incomingJob: Omit<Schema.Schema.To<typeof IncomingJob>, "input">
  ): Promise<void>;
}

function assertEnv(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`missing env var ${name}`);
  }
  return value;
}

const getDefaultPointgaurgBaseUrl = () => assertEnv("POINTGUARD_URL");
const getDefaultJobHandlerUrl = () => assertEnv("POINTGUARD_JOBS_URL");

export function defineJob<Input>(options: {
  handler: (input: Input) => Promise<void>;
  name: string;
  maxRetries?: number;
  pointguardBaseUrl?: URL;
  jobHandlerUrl?: URL;
}): Job<Input> {
  return {
    handler: options.handler,
    name: options.name,
    ...createChainedEnqueuer({
      pointguardBaseUrl: options.pointguardBaseUrl,
      jobHandlerUrl: options.jobHandlerUrl,
      jobName: options.name,
      opts: {},
    }),
  };
}

export const IncomingJob = Schema.struct({
  jobName: Schema.string,
  input: Schema.unknown,
  retryCount: Schema.number,
  maxRetries: Schema.number,
  createdAt: Schema.string.pipe(Schema.dateFromString),
}) satisfies Schema.Schema<components["schemas"]["InvokedTaskPayload"], any>;

export type DecodedIncomingJob = Schema.Schema.To<typeof IncomingJob>;
export type EncodedIncomingJob = Schema.Schema.From<typeof IncomingJob>;
export const parseIncomingJob = Schema.parseSync(IncomingJob);
