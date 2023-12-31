import type { paths } from "./generated/pointguard";
import { Fetcher } from "openapi-typescript-fetch";

const apiClient = Fetcher.for<paths>();
apiClient.configure({
  baseUrl: new URL("/", window.location.href).href.replace(/\/$/, ""),
  init: {
    headers: {
      "x-pointguard": "1",
    },
  },
});

export { apiClient };
