import { defineJob } from "@pointguard/nextjs";
import { setTimeout } from "timers/promises";

export const SayHello = defineJob({
  name: "my-job",
  async handler(input: { name: string }) {
    await setTimeout(5000);
    throw new Error("can't create hatraa because kakakasdjnajsdf");
  },
});
