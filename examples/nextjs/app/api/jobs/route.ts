import { createHandler } from "@pointguard/nextjs";
import { SayHello } from "./my-job";

export const POST = createHandler({
  jobs: [SayHello],
});
