import * as Schema from "@effect/schema/Schema";

const EnqueueOptions = Schema.struct({
  runAt: Schema.string.pipe(Schema.dateFromString, Schema.optional),
  maxRetries: Schema.number.pipe(
    Schema.greaterThanOrEqualTo(0),
    Schema.optional
  ),
  name: Schema.string.pipe(Schema.optional),
});

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

function getEnqueueUrl(baseUrl: string) {
  return `${baseUrl.replace(/\/$/, "")}/api/v1/tasks`;
}

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
        enqueueUrl: getEnqueueUrl(
          String(opts.pointguardBaseUrl || getDefaultPointgaurgBaseUrl())
        ),
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
  enqueueUrl,
  jobHandlerUrl,
}: {
  jobName: string;
  opts?: Schema.Schema.To<typeof EnqueueOptions>;
  input: unknown;
  jobHandlerUrl: URL | string | undefined;
  enqueueUrl: URL | string;
}) {
  const jobOptions = encodeEnqueueOptions(opts ?? {});
  const body = JSON.stringify({
    data: input,
    jobName,
    endpoint: String(jobHandlerUrl || getDefaultJobHandlerUrl()),
    ...jobOptions,
  });
  const response = await fetch(enqueueUrl, {
    cache: "no-cache",
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body,
  });
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
});

export type DecodedIncomingJob = Schema.Schema.To<typeof IncomingJob>;
export type EncodedIncomingJob = Schema.Schema.From<typeof IncomingJob>;
export const parseIncomingJob = Schema.parseSync(IncomingJob);
