import { paths } from "../generated/openapi";

export * from "../generated/openapi";

type BodyOf<
  P extends keyof paths,
  M extends keyof paths[P],
> = paths[P][M] extends {
  requestBody: { content: { "application/json": infer B } };
}
  ? B
  : void;

export class Client {
  constructor(
    private readonly options: { baseUrl: string; init?: RequestInit }
  ) {
    options.baseUrl = options.baseUrl.replace(/\/$/, "");
  }

  request = <Path extends keyof paths, Method extends keyof paths[Path]>(
    path: Path,
    method: Method,
    body: BodyOf<Path, Method>
  ) => {
    const url = this.options.baseUrl + path;
    const headers = new Headers(this.options.init?.headers);
    headers.set("Content-Type", "application/json");
    const init: RequestInit = {
      ...this.options.init,
      headers,
      method: String(method),
      body: JSON.stringify(body),
    };
    return new Request(url, init);
  };
}

export const createClient = (
  ...options: ConstructorParameters<typeof Client>
) => new Client(...options);
