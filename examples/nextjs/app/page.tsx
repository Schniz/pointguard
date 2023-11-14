import { SayHello } from "./api/jobs/my-job";

export default function Home() {
  return (
    <div>
      <form
        action={async (formData: FormData) => {
          "use server";

          await SayHello.enqueue(
            {
              name: formData.get("name") as string,
            },
            {
              // 1 hour from now
              // runAt: new Date(Date.now() + 1000 * 60 * 60),
            }
          );
        }}
      >
        <input type="text" name="name" />
        <input type="submit" />
      </form>
    </div>
  );
}
