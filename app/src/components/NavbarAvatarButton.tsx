"use client";

import {
  Avatar,
  Box,
  Flex,
  Icon,
  Popover,
  Portal,
  SkeletonCircle,
  Text,
  Separator as Divider,
  VStack,
} from "@chakra-ui/react";
import { LogOut, User2, Wallet } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { googleLogout } from "@react-oauth/google";
import jsCookie from "js-cookie";
import useUserInfo from "@/hooks/useUserInfo";
import useRevalidation from "@/hooks/useRevalidate";
import GoogleSignInButton from "./GoogleSignInButton";

const NavbarAvatarButton = () => {
  const { data, isLoading } = useUserInfo();
  const [openPopover, setOpenPopover] = useState(false);
  const revalidate = useRevalidation();

  function handleLogout() {
    jsCookie.remove("polymarketAuthToken");
    googleLogout();
    queueMicrotask(() => revalidate(["userData"]));
    setOpenPopover(false);
    window.location.reload();
  }

  // Mock balance data - replace with actual balance from your API
  const userBalance = Number(data?.balance || 30).toLocaleString("en-US", {
    style: "currency",
    currency: "USD",
  });

  if (isLoading) {
    return <SkeletonCircle size="9" />;
  }

  return (
    <div>
      {data && !isLoading ? (
        <Popover.Root
          size="sm"
          onOpenChange={(open) => setOpenPopover(open.open)}
          open={openPopover}
        >
          <Popover.Trigger>
            <Avatar.Root shape="full" size="sm">
              <Avatar.Fallback name={data.name} />
              <Avatar.Image src={data.avatar} />
            </Avatar.Root>
          </Popover.Trigger>
          <Portal>
            <Popover.Positioner>
              <Popover.Content minW="280px">
                <Popover.Body padding={0}>
                  <VStack gap={0} align="stretch">
                    {/* User Info Header */}
                    <Box p={4} bg="gradient-to-r from-blue.50 to-purple.50">
                      <Flex align="center" gap={3}>
                        <Avatar.Root size="md">
                          <Avatar.Fallback name={data.name} />
                          <Avatar.Image src={data.avatar} />
                        </Avatar.Root>
                        <Box flex={1}>
                          <Text
                            fontWeight="bold"
                            fontSize="md"
                            color="gray.800"
                          >
                            {data.name}
                          </Text>
                          <Text fontSize="sm" color="gray.600">
                            {data.email}
                          </Text>
                        </Box>
                      </Flex>
                    </Box>

                    {/* Balance Section */}
                    <Box p={4} bg="white">
                      <VStack gap={3} align="stretch">
                        <Flex justify="space-between" align="center">
                          <Flex align="center" gap={2}>
                            <Icon color="green.500">
                              <Wallet />
                            </Icon>
                            <Text
                              fontSize="sm"
                              fontWeight="medium"
                              color="gray.700"
                            >
                              Balance
                            </Text>
                          </Flex>
                          <Text
                            fontSize="lg"
                            fontWeight="bold"
                            color="green.600"
                          >
                            {userBalance}
                          </Text>
                        </Flex>
                      </VStack>
                    </Box>

                    <Divider />

                    {/* Action Items */}
                    <Box p={2}>
                      <VStack gap={1} align="stretch">
                        <Box
                          padding={2}
                          rounded="md"
                          _hover={{
                            backgroundColor: "gray.100",
                            cursor: "pointer",
                          }}
                          transition="all 0.2s"
                        >
                          <Link href="/profile" className="flex items-center">
                            <Icon size="md" color="gray.600">
                              <User2 />
                            </Icon>
                            <Text ml={3} fontSize="sm" fontWeight="medium">
                              My Profile
                            </Text>
                          </Link>
                        </Box>

                        <Flex
                          padding={2}
                          rounded="md"
                          _hover={{
                            backgroundColor: "red.50",
                            cursor: "pointer",
                          }}
                          onClick={handleLogout}
                          as="button"
                          width="full"
                          transition="all 0.2s"
                        >
                          <Icon size="md" color="red.500">
                            <LogOut />
                          </Icon>
                          <Text
                            ml={3}
                            color="red.500"
                            fontSize="sm"
                            fontWeight="medium"
                          >
                            Sign Out
                          </Text>
                        </Flex>
                      </VStack>
                    </Box>
                  </VStack>
                </Popover.Body>
              </Popover.Content>
            </Popover.Positioner>
          </Portal>
        </Popover.Root>
      ) : (
        <GoogleSignInButton />
      )}
    </div>
  );
};

export default NavbarAvatarButton;
