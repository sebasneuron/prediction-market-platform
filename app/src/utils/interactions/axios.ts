import { Axios } from "axios";

let axiosInstance: Axios;

function createAxiosInstance() {
  if (!axiosInstance) {
    axiosInstance = new Axios({
      baseURL: process.env.NEXT_PUBLIC_SERVICE_API_URL,
      headers: {
        "Content-Type": "application/json",
      },
      timeout: 10000, // 10 seconds timeout
    });
  }
  return axiosInstance;
}

axiosInstance = createAxiosInstance();

export { axiosInstance };
