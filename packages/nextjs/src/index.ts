import {
  Job,
  RejectedError,
  isRetriable,
  parseIncomingJob,
  webhooks,
} from "@pointguard/core";

type ReturnValue =
  webhooks["executeTask"]["post"]["responses"]["200"]["content"]["application/json"];

export {
  type DecodedIncomingJob,
  defineJob,
  type EncodedIncomingJob,
  IncomingJob,
  type Job,
} from "@pointguard/core";

async function handleIncomingJob(
  jobsByName: Map<string, Job<any>>,
  request: Request
): Promise<ReturnValue> {
  try {
    const body = await request
      .json()
      .catch((err) => {
        throw new Error(`failed to parse request body`, { cause: err });
      })
      .then(parseIncomingJob)
      .catch((err) => {
        throw new RejectedError(
          `request body doesn't match the expected format`,
          { cause: err }
        );
      });

    const job = jobsByName.get(body.jobName);
    if (!job) {
      throw new RejectedError(`job ${body.jobName} is not defined`);
    }

    await job.handler(body.input, {
      createdAt: body.createdAt,
      retryCount: body.retryCount,
      maxRetries: body.maxRetries,
      jobName: body.jobName,
    });

    return { success: {} };
  } catch (e) {
    return { failure: { reason: String(e), retriable: isRetriable(e) } };
  }
}

export function createHandler(opts: {
  jobs: Job<any>[];
}): (request: Request) => Promise<Response> {
  const jobsByName = new Map<string, Job<any>>();
  for (const job of opts.jobs) {
    jobsByName.set(job.name, job);
  }

  return async (request: Request) => {
    return Response.json(await handleIncomingJob(jobsByName, request));
  };
}
