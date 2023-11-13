import {
  RouterProvider,
  createBrowserRouter,
  redirect,
} from "react-router-dom";
import { Root } from "./root";

const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <Root />,
      children: [
        {
          path: "/enqueued",
          element: <div>Enqueued</div>,
        },
        {
          path: "/finished",
          element: <div>Finished</div>,
        },
        {
          path: "/",
          Component: null,
          loader: () => redirect("/enqueued"),
        },
      ],
    },
  ],
  {
    basename: import.meta.env.BASE_URL,
  }
);

function App() {
  return <RouterProvider router={router} />;
}

export default App;