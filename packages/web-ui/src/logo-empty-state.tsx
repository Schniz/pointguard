import logo from "./logo.png?url";

export function LogoEmptyState(props: React.PropsWithChildren) {
  return (
    <div className="items-center justify-center flex flex-col p-4">
      <img src={logo} className="w-48 opacity-75" />
      <div className="p-4 text-gray-500">{props.children}</div>
    </div>
  );
}
