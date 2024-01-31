import { useState } from "react";
import { apiClient } from "../api-client";
import formatRelative from "date-fns/formatRelative";
import { useSWR } from "../swr";
import { LogoEmptyState } from "../logo-empty-state";

type LoaderData = Awaited<ReturnType<typeof loader>>;

export function Component() {
  const [data, refetch] = useSWR<LoaderData>();

  return (
    <>
      {data.length === 0 ? (
        <LogoEmptyState>No enqueued tasks.</LogoEmptyState>
      ) : (
        <div className="max-w-screen-lg mx-auto mt-4 rounded-xl overflow-hidden w-full">
          {data.map((task) => (
            <Row task={task} key={task.id} refetch={refetch} />
          ))}
        </div>
      )}
    </>
  );
}

function Row({ task, refetch }: { task: LoaderData[number]; refetch(): void }) {
  const [open, setOpen] = useState(false);

  return (
    <div
      className="even:bg-gray-50 bg-white"
      tabIndex={0}
      role="checkbox"
      onClick={() => setOpen((o) => !o)}
    >
      <div className="flex items-center text-gray-500 text-sm p-4 space-x-4">
        <div className="text-gray-400 text-sm flex items-center">
          <div
            data-active={!!task.workerId}
            className="ml-2 rounded-full h-2 w-2 data-[active=true]:bg-orange-400 data-[active=true]:block hidden"
          />
        </div>
        <div>
          <svg
            className="w-6 h-6 text-gray-400"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            {!open ? (
              <path
                fillRule="evenodd"
                d="M10 13a1 1 0 01-.707-.293l-3-3a1 1 0 111.414-1.414L10 10.586l2.293-2.293a1 1 0 111.414 1.414l-3 3A1 1 0 0110 13z"
                clipRule="evenodd"
              />
            ) : (
              <path
                fillRule="evenodd"
                d="M10 7a1 1 0 01.707.293l3 3a1 1 0 11-1.414 1.414L10 9.414 7.707 11.707a1 1 0 11-1.414-1.414l3-3A1 1 0 0110 7z"
                clipRule="evenodd"
              />
            )}
          </svg>
        </div>
        <div>
          <time dateTime={task.runAt} x-timeago="runAt">
            {formatRelative(new Date(task.runAt), new Date())}
          </time>
        </div>
        <div className="whitespace-nowrap text-sm text-gray-500">
          <div className="font-medium">{task.jobName}</div>
          <div className="text-gray-400">{task.endpoint}</div>
        </div>
        <div className="font-mono text-xs break-all">
          {Boolean(task.retryCount > 0 && task.maxRetries) && (
            <>
              {task.retryCount}/{task.maxRetries}
            </>
          )}
        </div>
        <div className="flex-1 text-xs break-all text-end flex items-center justify-end">
          <button
            onClick={(e) => {
              e.stopPropagation();

              if (!confirm("Are you sure you want to unshift this task?")) {
                return;
              }

              apiClient
                .path("/api/v1/tasks/{id}/unshift")
                .method("post")
                .create()({
                  id: task.id,
                })
                .finally(refetch);
            }}
            type="submit"
            className="inline-block py-2 px-3 rounded-xl hover:underline decoration-orange-400 text-orange-500"
          >
            Unshift
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();

              if (!confirm("Are you sure you want to cancel this task?")) {
                return;
              }

              apiClient
                .path("/api/v1/tasks/{id}/cancel")
                .method("post")
                .create()({
                  id: task.id,
                })
                .finally(refetch);
            }}
            type="submit"
            className="inline-block py-2 px-3 rounded-xl hover:underline decoration-red-400 text-red-500"
          >
            Cancel
          </button>
        </div>
      </div>
      {open && (
        <div
          className="p-4 text-xs text-gray-500"
          onClick={(e) => e.stopPropagation()}
        >
          <pre>{JSON.stringify(task.data)}</pre>
        </div>
      )}
    </div>
  );
}

export const loader = async () => {
  return await apiClient
    .path("/api/v1/tasks/enqueued")
    .method("get")
    .create()({})
    .then((x) => x.data);
};
