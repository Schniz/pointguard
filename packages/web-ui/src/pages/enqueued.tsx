import { useCallback, useEffect, useRef, useState } from "react";
import { apiClient } from "../api-client";
import { useLoaderData, useLocation, useNavigate } from "react-router-dom";
import formatRelative from "date-fns/formatRelative";

function useSWR<T>(): [T, refetch: () => void] {
  const data = useLoaderData() as T;
  const l = useLocation();
  const navigate = useNavigate();

  const callback = useCallback(() => navigate(location.current), [navigate]);

  const location = useRef(l);
  useEffect(() => {
    location.current = l;
  }, [l]);

  useEffect(() => {
    const interval = setInterval(callback, 5000);
    window.addEventListener("focus", callback);

    return () => {
      window.removeEventListener("focus", callback);
      clearInterval(interval);
    };
  }, [navigate, callback]);

  return [data, callback];
}

type LoaderData = Awaited<ReturnType<typeof loader>>;

export function Component() {
  const [data, refetch] = useSWR<LoaderData>();

  return (
    <div id="enqueued-table" className="divide-y divide-gray-200 bg-white">
      {data.map((task) => (
        <Row task={task} key={task.id} refetch={refetch} />
      ))}
    </div>
  );
}

function Row({ task, refetch }: { task: LoaderData[number]; refetch(): void }) {
  const [open, setOpen] = useState(false);

  return (
    <div
      className="even:bg-orange-50 even:bg-opacity-50"
      tabIndex={0}
      role="checkbox"
      onClick={() => setOpen((o) => !o)}
    >
      <div className="flex items-center text-gray-500 text-sm p-4 space-x-2">
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
        <div className="text-gray-400 text-sm flex items-center">
          <div
            data-active={!!task.workerId}
            className="ml-2 rounded-full h-2 w-2 data-[active=true]:bg-orange-400 data-[active=true]:block hidden"
          ></div>
        </div>
        <div className="whitespace-nowrap text-sm text-gray-500">
          {task.jobName}
        </div>
        <div className="text-gray-500 break-all font-mono">{task.name}</div>
        <div className="font-mono text-xs break-all">
          {Boolean(task.retryCount > 0 && task.maxRetries) && (
            <>
              {task.retryCount}/{task.maxRetries}
            </>
          )}
        </div>
        <div>
          <time dateTime={task.runAt} x-timeago="runAt">
            {formatRelative(new Date(task.runAt), new Date())}
          </time>
        </div>
        <div className="flex-1 text-xs break-all text-end flex items-center justify-end">
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
        <div className="p-4 text-xs text-gray-500">
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
