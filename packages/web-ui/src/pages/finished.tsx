import intervalToDuration from "date-fns/intervalToDuration";
import { apiClient } from "../api-client";
import { useTypedLoaderData } from "../swr";
import formatDuration from "date-fns/formatDuration";
import { useState } from "react";
import { LogoEmptyState } from "../logo-empty-state";
import { Link, LoaderFunctionArgs, useSearchParams } from "react-router-dom";
import { twMerge } from "tailwind-merge";
import {
  LucideStepForward,
  LucideStepBack,
  LucideChevronDown,
  LucideChevronUp,
} from "lucide-react";

type LoaderData = Awaited<ReturnType<typeof loader>>;

export function Component() {
  const { totalPages, page, items } = useTypedLoaderData<LoaderData>();
  return (
    <>
      {items.length === 0 ? (
        <LogoEmptyState>No finished tasks.</LogoEmptyState>
      ) : (
        <div className="max-w-screen-lg w-full mx-auto my-4 rounded-xl overflow-hidden">
          <div className="grid grid-cols-[repeat(3,min-content)_1fr_min-content]">
            {items.map((task, i) => (
              <Row index={i} task={task} key={task.id} />
            ))}
          </div>
          <Pagination totalPages={totalPages} page={page} />
        </div>
      )}
    </>
  );
}

function withValues(
  searchParams: URLSearchParams,
  newValues: [string, string][]
) {
  const newSearchParams = new URLSearchParams(searchParams.toString());

  for (const [key, value] of newValues) {
    newSearchParams.set(key, value);
  }

  return newSearchParams;
}

function Pagination(props: { totalPages: number; page: number }) {
  const [searchParams] = useSearchParams();
  const previousStyle =
    "block text-sm px-2 font-medium text-white data-[disabled=true]:cursor-not-allowed data-[disabled=true]:opacity-30 py-2 py-2 flex space-x-1 items-center aria-current:underline underline-offset-4 hover:underline hover:decoration-orange-300 hover:text-orange-200";
  const iconStyle = "block w-4 h-4";

  return (
    <div className="flex items-center justify-center bg-orange-500 px-4 py-2">
      <Link
        className={previousStyle}
        data-disabled={props.page <= 1}
        to={{
          search: `?${withValues(searchParams, [
            ["page", String(Math.max(props.page - 1, 1))],
          ])}`,
        }}
      >
        <LucideStepBack className={iconStyle} />
      </Link>
      {Array.from({ length: props.totalPages }).map((_, i) => {
        const page = i + 1;
        return (
          <Link
            {...{ "aria-current": props.page === page ? "page" : undefined }}
            key={page}
            className={twMerge(previousStyle, "tabular-nums")}
            to={{
              search: `?${withValues(searchParams, [["page", String(page)]])}`,
            }}
          >
            {page}
          </Link>
        );
      })}
      <Link
        className={previousStyle}
        data-disabled={props.page >= props.totalPages}
        to={{
          search: `?${withValues(searchParams, [
            ["page", String(Math.min(props.page + 1, props.totalPages))],
          ])}`,
        }}
      >
        <LucideStepForward className={iconStyle} />
      </Link>
    </div>
  );
}

function Row({
  task,
  index,
}: {
  task: LoaderData["items"][number];
  index: number;
}) {
  const duration =
    formatDuration(
      intervalToDuration({
        start: new Date(task.startedAt),
        end: new Date(task.createdAt),
      })
    ) || "blazing";
  const [open, setOpen] = useState(false);

  const backgroundStyle = twMerge(
    "bg-white text-gray-900 px-4 py-4 flex items-center text-sm",
    index % 2 === 0 && "bg-gray-50"
  );

  return (
    <>
      <div className={twMerge(backgroundStyle, "p-0")}>
        <button
          className="block flex-1 h-full px-2 pl-4 group"
          onClick={() => setOpen((o) => !o)}
          data-open={open}
        >
          <LucideChevronDown className="w-4 h-4 group-data-[open=true]:hidden" />
          <LucideChevronUp className="w-4 h-4 group-data-[open=false]:hidden" />
        </button>
      </div>
      <div
        className={twMerge(
          backgroundStyle,
          "whitespace-nowrap text-gray-600 block"
        )}
      >
        <div className="font-medium">{task.jobName}</div>
        <div className="text-gray-400">{task.endpoint}</div>
      </div>
      <div
        className={twMerge(
          backgroundStyle,
          "whitespace-nowrap text-gray-600 flex flex-col justify-center"
        )}
      >
        <div>{duration}</div>
        {task.retries > 0 && (
          <div className="text-gray-400">
            {task.retries === 1 ? "retried once" : `${task.retries} retries`}
          </div>
        )}
      </div>
      <div
        className={twMerge(backgroundStyle, "font-mono text-red-500 text-xs")}
      >
        {task.errorMessage}
      </div>
      <div className={twMerge(backgroundStyle, "text-gray-300 tabular-nums")}>
        #{task.id}
      </div>
      {open && (
        <pre
          className={twMerge(
            backgroundStyle,
            "col-span-5 text-gray-500 font-mono"
          )}
        >
          {JSON.stringify(task.data, null, 2)}
        </pre>
      )}
    </>
  );
}

export const loader = async (args: LoaderFunctionArgs) => {
  const { searchParams } = new URL(args.request.url);
  const limit = Number(searchParams.get("limit")) || 10;
  const page = Number(searchParams.get("page")) || null;

  const result = await apiClient
    .path("/api/v1/tasks/finished")
    .method("get")
    .create()({
      page,
      limit,
    })
    .then((x) => x.data);

  return result;
};
