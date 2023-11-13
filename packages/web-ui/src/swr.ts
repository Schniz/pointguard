import { useCallback, useEffect, useRef } from "react";
import { useLoaderData, useLocation, useNavigate } from "react-router-dom";

export function useSWR<T>(): [T, refetch: () => void] {
  const data = useLoaderData() as T;
  const l = useLocation();
  const navigate = useNavigate();

  const callback = useCallback(() => navigate(location.current), [navigate]);

  const location = useRef(l);
  useEffect(() => {
    location.current = l;
  }, [l]);

  useEffect(() => {
    const interval = setInterval(callback, 5000);
    window.addEventListener("focus", callback);

    return () => {
      window.removeEventListener("focus", callback);
      clearInterval(interval);
    };
  }, [navigate, callback]);

  return [data, callback];
}
