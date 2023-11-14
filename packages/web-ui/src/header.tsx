import { twMerge } from "tailwind-merge";
import { NavLink as RRNavLink } from "react-router-dom";
import logo from "./logo.png?url";

function NavLink(props: React.ComponentProps<typeof RRNavLink>) {
  return (
    <RRNavLink
      {...props}
      className={({ isActive }) =>
        twMerge(
          "py-2 px-4 block decoration-orange-400 hover:underline hover:text-orange-500",
          isActive && "underline font-bold"
        )
      }
    />
  );
}

export function Header() {
  return (
    <nav className="flex items-center justify-between">
      <div className="pointer-events-none text-sm block px-4 py-2 space-x-2">
        <img src={logo} className="w-6 h-6 inline-block -mt-1" />
        <span className="font-bold">Pointguard</span>
      </div>
      <div className="flex items-center">
        <NavLink to="/enqueued">Enqueued</NavLink>
        <NavLink to="/finished">Finished</NavLink>
      </div>
    </nav>
  );
}
