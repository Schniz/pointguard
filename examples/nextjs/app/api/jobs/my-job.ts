import { defineJob } from "@pointguard/nextjs";
import { setTimeout } from "timers/promises";

export const SayHello = defineJob({
  name: "my-job",
  async handler(input: { name: string }) {
    await setTimeout(20000);
    console.log(`Hello ${input.name}!`);
  },
});
