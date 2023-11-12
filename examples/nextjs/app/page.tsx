import { SayHello } from "./api/jobs/my-job";

export default function Home() {
  return (
    <div>
      <form
        action={async (formData: FormData) => {
          "use server";

          await SayHello.enqueue({
            name: formData.get("name") as string,
          });
        }}
      >
        <input type="text" name="name" />
        <input type="submit" />
      </form>
    </div>
  );
}
