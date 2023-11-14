export function EmptyState<T>(props: {
  data: T[];
  emptyState: React.ReactNode;
  children(t: T): React.ReactNode;
}) {
  if (props.data.length) {
    return props.data.map(props.children);
  }
  return props.emptyState;
}
