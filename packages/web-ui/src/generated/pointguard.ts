/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */


/** OneOf type helpers */
type Without<T, U> = { [P in Exclude<keyof T, keyof U>]?: never };
type XOR<T, U> = (T | U) extends object ? (Without<T, U> & U) | (Without<U, T> & T) : T | U;
type OneOf<T extends any[]> = T extends [infer Only] ? Only : T extends [infer A, infer B, ...infer Rest] ? OneOf<[XOR<A, B>, ...Rest]> : never;

export interface paths {
  "/api/v1/events": {
    /**
     * /api/v1/events
     * @description get realtime events. This is a server sent event stream, but I can't figure out how to document it in openapi.
     */
    get: {
      responses: {
        200: {
          content: {
            "application/json": OneOf<[{
              /** @enum {string} */
              type: "taskEnqueued";
            }, {
              /** @enum {string} */
              type: "taskInvoked";
            }, {
              /** @enum {string} */
              type: "taskFailed";
            }, {
              /** @enum {string} */
              type: "taskFinished";
            }]>;
          };
        };
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
  "/api/v1/version": {
    get: {
      responses: {
        /** @description plain text */
        200: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
  "/api/v1/tasks": {
    post: {
      requestBody: {
        content: {
          "application/json": {
            /** @description The data that will be passed on execution. */
            data?: unknown;
            /**
             * Format: uri
             * @description The pointguard endpoint that'll be invoked
             */
            endpoint: string;
            /** @description The job name. This is used to know which function to invoke. */
            jobName: string;
            /** Format: uint */
            maxRetries?: number | null;
            /** @description A name for the task. If not provided, a random name will be generated. This is useful to throttle tasks of the same type. */
            name?: string | null;
            /**
             * Format: date-time
             * @description When to run the task. If not provided, it'll run as soon as possible.
             */
            runAt?: string | null;
          };
        };
      };
      responses: {
        200: {
          content: {
            "application/json": number;
          };
        };
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
  "/api/v1/tasks/{id}/cancel": {
    post: {
      parameters: {
        path: {
          id: number;
        };
      };
      responses: {
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
  "/api/v1/tasks/{id}/unshift": {
    post: {
      parameters: {
        path: {
          id: number;
        };
      };
      responses: {
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
  "/api/v1/tasks/enqueued": {
    get: {
      responses: {
        200: {
          content: {
            "application/json": ({
                /** Format: date-time */
                createdAt: string;
                data: unknown;
                endpoint: string;
                /** Format: int64 */
                id: number;
                jobName: string;
                /** Format: int32 */
                maxRetries: number;
                name: string;
                /** Format: int32 */
                retryCount: number;
                /** Format: date-time */
                runAt: string;
                workerId?: string | null;
              })[];
          };
        };
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
  "/api/v1/tasks/finished": {
    get: {
      parameters: {
        query?: {
          limit?: number | null;
          page?: number | null;
        };
      };
      responses: {
        200: {
          content: {
            "application/json": {
              items: ({
                  /** Format: date-time */
                  createdAt: string;
                  data: unknown;
                  endpoint: string;
                  errorMessage?: string | null;
                  /** Format: int64 */
                  id: number;
                  jobName: string;
                  name: string;
                  /** Format: int32 */
                  retries: number;
                  /** Format: date-time */
                  startedAt: string;
                })[];
              /** Format: uint */
              page: number;
              /** Format: uint */
              totalPages: number;
            };
          };
        };
        /** @description plain text */
        default: {
          content: {
            "text/plain; charset=utf-8": unknown;
          };
        };
      };
    };
  };
}

export interface webhooks {
  "executeTask": {
    post: {
      requestBody: {
        content: {
          "application/json": {
            /**
             * Format: date-time
             * @description The time when this task was enqueued at
             */
            createdAt: string;
            /** @description The input data of the task */
            input: unknown;
            /** @description The job name to invoke */
            jobName: string;
            /**
             * Format: int32
             * @description The maximum amount of times we can retry this task
             */
            maxRetries: number;
            /**
             * Format: int32
             * @description The amount of times we retried this task
             */
            retryCount: number;
          };
        };
      };
      responses: {
        200: {
          content: {
            "application/json": OneOf<[{
              success: Record<string, never>;
            }, {
              failure: {
                /** @description The reason why it failed */
                reason: string;
                /**
                 * @description Whether or not this task is retriable
                 * @default true
                 */
                retriable?: boolean;
              };
            }]>;
          };
        };
      };
    };
  };
}

export interface components {
  schemas: {
    /** InvokedTaskPayload */
    InvokedTaskPayload: {
      /**
       * Format: date-time
       * @description The time when this task was enqueued at
       */
      createdAt: string;
      /** @description The input data of the task */
      input: unknown;
      /** @description The job name to invoke */
      jobName: string;
      /**
       * Format: int32
       * @description The maximum amount of times we can retry this task
       */
      maxRetries: number;
      /**
       * Format: int32
       * @description The amount of times we retried this task
       */
      retryCount: number;
    };
  };
  responses: never;
  parameters: never;
  requestBodies: never;
  headers: never;
  pathItems: never;
}

export type $defs = Record<string, never>;

export type external = Record<string, never>;

export type operations = Record<string, never>;
