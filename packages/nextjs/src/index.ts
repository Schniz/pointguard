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

function createChainedEnqueuer<Input>(opts: {
  jobName: string;
  enqueueUrl: URL | string;
  jobHandlerUrl: URL | string;
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
        enqueueUrl: opts.enqueueUrl,
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
  jobHandlerUrl: URL | string;
  enqueueUrl: URL | string;
}) {
  const jobOptions = encodeEnqueueOptions(opts ?? {});
  const body = JSON.stringify({
    data: input,
    jobName,
    endpoint: String(jobHandlerUrl),
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

interface Job<Input> extends ChainedEnqueuer<Input> {
  name: string;

  handler(
    input: Input,
    incomingJob: Omit<Schema.Schema.To<typeof IncomingJob>, "input">
  ): Promise<void>;
}

export function defineJob<Input>(options: {
  handler: (input: Input) => Promise<void>;
  name: string;
  maxRetries?: number;
  pointguardBaseUrl?: URL;
  jobHandlerUrl?: URL;
}): Job<Input> {
  const pointguardBaseUrl =
    options.pointguardBaseUrl ?? process.env.POINTGUARD_URL;
  if (!pointguardBaseUrl) {
    throw new Error("pointguardBaseUrl is required (or POINTGUARD_URL)");
  }
  const jobHandlerUrl =
    options.jobHandlerUrl ?? process.env.POINTGUARD_JOBS_URL;
  if (!jobHandlerUrl) {
    throw new Error("jobHandlerUrl is required (or POINTGUARD_JOBS_URL)");
  }
  const enqueueUrl = new URL(pointguardBaseUrl);
  enqueueUrl.pathname = `${enqueueUrl.pathname.replace(
    /\/$/,
    ""
  )}/api/v1/tasks`;
  return {
    handler: options.handler,
    name: options.name,
    ...createChainedEnqueuer({
      enqueueUrl: String(enqueueUrl),
      jobHandlerUrl: String(jobHandlerUrl),
      jobName: options.name,
      opts: {},
    }),
  };
}

const IncomingJob = Schema.struct({
  jobName: Schema.string,
  input: Schema.unknown,
  retryCount: Schema.number,
  maxRetries: Schema.number,
  createdAt: Schema.string.pipe(Schema.dateFromString),
});

const parseIncomingJob = Schema.parseSync(IncomingJob);

export function createHandler(opts: {
  jobs: Job<any>[];
}): (request: Request) => Promise<Response> {
  const jobsByName = opts.jobs.reduce((acc, job) => {
    acc.set(job.name, job);
    return acc;
  }, new Map<string, Job<any>>());

  return async (request: Request) => {
    const body = await request.json().then(parseIncomingJob);

    const job = jobsByName.get(body.jobName);
    if (!job) {
      return new Response("job not found", { status: 404 });
    }

    await job.handler(body.input, {
      createdAt: body.createdAt,
      retryCount: body.retryCount,
      maxRetries: body.maxRetries,
      jobName: body.jobName,
    });

    return new Response("ok");
  };
}
