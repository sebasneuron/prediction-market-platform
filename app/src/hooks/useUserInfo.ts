import { useQuery } from "@tanstack/react-query";

import { UserGetters } from "@/utils/interactions/dataGetter";

export default function useUserInfo() {
  const { data, isLoading } = useQuery({
    queryKey: ["userData"],
    queryFn: UserGetters.getUserData,
  });
  return {
    data,
    isLoading,
  };
}
