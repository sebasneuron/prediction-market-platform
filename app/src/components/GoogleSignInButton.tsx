import { GoogleLogin } from "@react-oauth/google";
import { useMutation } from "@tanstack/react-query";
import React from "react";
import cookie from "js-cookie";

import { UserAuthActions } from "@/utils/interactions/dataPosters";
import useRevalidation from "@/hooks/useRevalidate";
import { toaster } from "./ui/toaster";

const GoogleSignInButton = () => {
  const { mutateAsync } = useMutation({
    mutationFn: UserAuthActions.handleSignInWithGoogle,
  });
  const revalidate = useRevalidation();

  function handleLogin(loginId: string) {
    toaster.promise(mutateAsync({ id_token: loginId }), {
      error(arg: any) {
        return {
          title: "Error",
          description: arg?.message || "Failed to login with google",
        };
      },
      success(arg) {
        cookie.set("polymarketAuthToken", arg.sessionToken, {
          expires: 60 * 60 * 24 * 30, // 30 days,
          secure: true,
        });
        queueMicrotask(() => revalidate(["userData"]));
        window.location.reload();

        return {
          title: "Success",
          description: "Welcome to polymarket",
        };
      },
      loading: {
        title: "Waiting for sign in...",
        description: "Please complete your sign in process in popup window",
      },
    });
  }
  return (
    <>
      <GoogleLogin
        onSuccess={(credentialResponse) => {
          if (!credentialResponse.credential) {
            toaster.error({ title: "Failed to get credentials from google" });
            return;
          }
          handleLogin(credentialResponse.credential);
        }}
        onError={() => {
          console.log("Login Failed");
          toaster.error({ title: "Failed to login with google" });
        }}
        logo_alignment="center"
        shape="circle"
        size="large"
      />
    </>
  );
};

export default GoogleSignInButton;
