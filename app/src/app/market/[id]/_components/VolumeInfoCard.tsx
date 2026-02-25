import { VolumeInfo } from "@/generated/grpc_service_types/markets";
import { Flex, HoverCard, Icon, Portal, Table, Text } from "@chakra-ui/react";
import { Info } from "lucide-react";

type Props = {
  volumeInfo: VolumeInfo;
};

const VolumeInfoCard = ({ volumeInfo }: Props) => {
  const totalVolume = getTotalVolumeFromVolumeInfo(volumeInfo);
  return (
    <div>
      <HoverCard.Root
        size="sm"
        openDelay={100}
        positioning={{ placement: "right-end" }}
      >
        <HoverCard.Trigger asChild>
          <Flex alignItems="center">
            <Text
              color="gray.600"
              fontSize="sm"
              cursor="default"
              userSelect="none"
            >
              {totalVolume} Vol.
            </Text>
            <Icon ml={1}>
              <Info size={14} />
            </Icon>
          </Flex>
        </HoverCard.Trigger>
        <Portal>
          <HoverCard.Positioner>
            <HoverCard.Content>
              <HoverCard.Arrow />
              <Text fontWeight="bold" mb={2}>
                Volume Breakdown
              </Text>
              <Table.Root>
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeader fontWeight="bold">
                      Type
                    </Table.ColumnHeader>
                    <Table.ColumnHeader fontWeight="bold">
                      Qty
                    </Table.ColumnHeader>
                    <Table.ColumnHeader fontWeight="bold">
                      USD
                    </Table.ColumnHeader>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  <Table.Row>
                    <Table.Cell fontWeight="bold">Yes Buy</Table.Cell>
                    <Table.Cell color={"green.600"} fontWeight="bold">
                      {volumeInfo.yesBuyQty.toLocaleString()}
                    </Table.Cell>
                    <Table.Cell color={"green.600"} fontWeight="bold">
                      {volumeInfo.yesBuyUsd.toLocaleString("en-US", {
                        style: "currency",
                        currency: "USD",
                      })}
                    </Table.Cell>
                  </Table.Row>
                  <Table.Row>
                    <Table.Cell fontWeight="bold">Yes Sell</Table.Cell>
                    <Table.Cell color={"green.600"} fontWeight="bold">
                      {volumeInfo.yesSellQty.toLocaleString()}
                    </Table.Cell>
                    <Table.Cell color={"green.600"} fontWeight="bold">
                      {volumeInfo.yesSellUsd.toLocaleString("en-US", {
                        style: "currency",
                        currency: "USD",
                      })}
                    </Table.Cell>
                  </Table.Row>
                  <Table.Row>
                    <Table.Cell fontWeight="bold">No Buy</Table.Cell>
                    <Table.Cell color={"red.600"} fontWeight="bold">
                      {volumeInfo.noBuyQty.toLocaleString()}
                    </Table.Cell>
                    <Table.Cell color={"red.600"} fontWeight="bold">
                      {volumeInfo.noBuyUsd.toLocaleString("en-US", {
                        style: "currency",
                        currency: "USD",
                      })}
                    </Table.Cell>
                  </Table.Row>
                  <Table.Row>
                    <Table.Cell fontWeight="bold">No Sell</Table.Cell>
                    <Table.Cell color={"red.600"} fontWeight="bold">
                      {volumeInfo.noSellQty.toLocaleString()}
                    </Table.Cell>
                    <Table.Cell color={"red.600"} fontWeight="bold">
                      {volumeInfo.noSellUsd.toLocaleString("en-US", {
                        style: "currency",
                        currency: "USD",
                      })}
                    </Table.Cell>
                  </Table.Row>
                </Table.Body>
              </Table.Root>
            </HoverCard.Content>
          </HoverCard.Positioner>
        </Portal>
      </HoverCard.Root>
    </div>
  );
};

export default VolumeInfoCard;

function getTotalVolumeFromVolumeInfo(volumeInfo: VolumeInfo): string {
  const totalVolume =
    volumeInfo.noBuyUsd +
    volumeInfo.noSellUsd +
    volumeInfo.yesBuyUsd +
    volumeInfo.yesSellUsd;
  return totalVolume.toLocaleString("en-US", {
    style: "currency",
    currency: "USD",
  });
}
