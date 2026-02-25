import {
  IconButton,
  Input,
  Popover,
  Portal,
  Separator,
  Text,
} from "@chakra-ui/react";
import { Bell } from "lucide-react";
import React from "react";
import EmptyStateCustom from "./EmptyStateCustom";

const NavbarNotificationButton = () => {
  return (
    <Popover.Root>
      <Popover.Trigger>
        <IconButton variant="subtle" as="span">
          <Bell size={20} />
        </IconButton>
      </Popover.Trigger>

      <Portal>
        <Popover.Positioner>
          <Popover.Content>
            <Popover.Arrow />
            <Popover.Body>
              <Popover.Title fontWeight="semibold">
                Unread notifications
              </Popover.Title>
              <Separator />
              <EmptyStateCustom
                title="No notifications"
                description="You have no unread notifications at the moment."
              />
            </Popover.Body>
          </Popover.Content>
        </Popover.Positioner>
      </Portal>
    </Popover.Root>
  );
};

export default NavbarNotificationButton;
