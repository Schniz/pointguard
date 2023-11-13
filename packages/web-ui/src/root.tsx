import { Outlet } from "react-router-dom";
import { Header } from "./header";

export function Root() {
  return (
    <>
      <Header />
      <Outlet />
    </>
  );
}
