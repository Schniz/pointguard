import { Job, parseIncomingJob } from "@pointguard/core";

export {
  type DecodedIncomingJob,
  defineJob,
  type EncodedIncomingJob,
  IncomingJob,
  type Job,
} from "@pointguard/core";

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
