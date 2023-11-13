import { twMerge } from "tailwind-merge";
import { NavLink as RRNavLink } from "react-router-dom";

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
    <nav
      className="flex items-center justify-center relative"
      x-data="{ url: new URL(window.location.href) }"
    >
      <span className="pointer-events-none text-sm absolute left-2 top-1/2 -translate-y-1/2 opacity-75">
        🏀
        <span className="font-bold">Pointguard</span>
      </span>
      <NavLink to="/enqueued">Enqueued</NavLink>
      <NavLink to="/finished">Finished</NavLink>
    </nav>
  );
}
