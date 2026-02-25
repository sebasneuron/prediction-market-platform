import { Box, EmptyState, VStack } from "@chakra-ui/react";
import { Scroll } from "lucide-react";
import React from "react";

type Props = {
  title?: string;
  description?: string;
  actionButton?: React.ReactNode;
};

const EmptyStateCustom = ({ description, title, actionButton }: Props) => {
  return (
    <div>
      <EmptyState.Root>
        <EmptyState.Content>
          <EmptyState.Indicator>
            <Scroll />
          </EmptyState.Indicator>
          <VStack textAlign="center">
            <EmptyState.Title>{title}</EmptyState.Title>
            <EmptyState.Description>{description}</EmptyState.Description>
            <Box mt={4}>{actionButton}</Box>
          </VStack>
        </EmptyState.Content>
      </EmptyState.Root>
    </div>
  );
};

export default EmptyStateCustom;
