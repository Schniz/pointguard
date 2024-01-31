import { SayHello } from "./api/jobs/my-job";

export default function Home() {
  return (
    <div>
      <form
        action={async (formData: FormData) => {
          "use server";

          await SayHello.enqueue(
            {
              name: `${formData.get("name") as string}`,
            },
            {
              runAt: new Date(Date.now() + 1000 * 60 * 5),
            },
          );

          console.log("enqueued");
        }}
      >
        <input type="text" name="name" />
        <input type="submit" />
      </form>
    </div>
  );
}
