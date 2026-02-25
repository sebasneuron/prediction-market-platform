import { useQueryClient } from "@tanstack/react-query";

export default function useRevalidation() {
  const queryClient = useQueryClient();

  function revalidateKeys(keys: string[] | string) {
    (async function () {
      try {
        await queryClient.refetchQueries({
          queryKey: Array.isArray(keys) ? keys : [keys],
        });
        await queryClient.invalidateQueries({
          queryKey: Array.isArray(keys) ? keys : [keys],
        });
        // console.log("Queries refetched successfully:", keys);
      } catch (error) {
        console.error("Error refetching queries:", error);
        throw error;
      }
    })();
  }

  return revalidateKeys;
}
