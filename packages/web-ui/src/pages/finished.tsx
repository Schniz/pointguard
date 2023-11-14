import intervalToDuration from "date-fns/intervalToDuration";
import { apiClient } from "../api-client";
import { useSWR } from "../swr";
import formatDuration from "date-fns/formatDuration";
import { useState } from "react";
import { LogoEmptyState } from "../logo-empty-state";

type LoaderData = Awaited<ReturnType<typeof loader>>;

export function Component() {
  const [data] = useSWR<LoaderData>();
  return (
    <>
      {data.length === 0 ? (
        <LogoEmptyState>No finished tasks.</LogoEmptyState>
      ) : (
        <div className="max-w-screen-lg w-full mx-auto mt-4 rounded-xl overflow-hidden">
          {data.map((task) => (
            <Row task={task} key={task.id} />
          ))}
        </div>
      )}
    </>
  );
}

function Row({ task }: { task: LoaderData[number] }) {
  const duration =
    formatDuration(
      intervalToDuration({
        start: new Date(task.startedAt),
        end: new Date(task.createdAt),
      })
    ) || "blazing";
  const [open, setOpen] = useState(false);

  return (
    <div
      className="even:bg-gray-100 bg-white"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          setOpen((o) => !o);
        }
      }}
      role="checkbox"
      onClick={() => setOpen((o) => !o)}
    >
      <div className="p-4 flex space-x-4 items-center">
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
        <div className="text-sm">
          <div className="text-gray-500 font-medium">{task.jobName}</div>
          <div className="text-gray-400">{task.endpoint}</div>
        </div>
        <div>
          <div className="text-gray-500 text-sm">{duration}</div>
          {Boolean(task.retries) && (
            <div className="text-gray-400 text-sm">
              {task.retries === 1
                ? "retried one time"
                : `retries ${task.retries} times`}
            </div>
          )}
        </div>
        <div className="text-red-500 font-mono text-xs">
          {task.errorMessage}
        </div>
      </div>
      {open && (
        <div className="p-4 text-sm text-gray-500">
          <pre>{JSON.stringify(task.data, null, 2)}</pre>
        </div>
      )}
    </div>
  );
}

export const loader = () =>
  apiClient
    .path("/api/v1/tasks/finished")
    .method("get")
    .create()({})
    .then((x) => x.data);
