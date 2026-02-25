"use client";

import React from "react";
import { Search } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { Flex, Input, InputGroup, Text } from "@chakra-ui/react";

import NavbarAvatarButton from "./NavbarAvatarButton";
import NavbarNotificationButton from "./NavbarNotificationButton";

const Navbar = () => {
  return (
    <Flex
      padding={4}
      justifyContent="space-between"
      borderBottom="1px solid"
      borderColor="gray.200"
      alignItems="center"
    >
      {/* left side */}
      <Flex alignItems="center">
        <Link href="/">
          <Image src="/assets/logo.svg" alt="Logo" width={135} height={23} />
        </Link>
        {/* links */}
        <Flex as="nav" ml={8} gap={6} display={["none", "flex"]}>
          {LINKS.map((link) => (
            <Link href={link.href} key={link.name}>
              <Text
                fontSize="14"
                fontWeight="medium"
                color="gray.700"
                _hover={{ textDecoration: "underline" }}
              >
                {link.name}
              </Text>
            </Link>
          ))}
        </Flex>
      </Flex>

      {/* right section */}
      <Flex gap={4}>
        <InputGroup
          startElement={<Search opacity={0.4} />}
          display={["none", "flex"]}
        >
          <Input placeholder="Search" variant="subtle" />
        </InputGroup>
        <NavbarNotificationButton />
        <NavbarAvatarButton />
      </Flex>
    </Flex>
  );
};

export default Navbar;

const LINKS = [
  {
    name: "Home",
    href: "/",
  },
  // {
  //   name: "Browse",
  //   href: "/browse",
  // },
  {
    name: "Profile",
    href: "/profile",
  },
];
