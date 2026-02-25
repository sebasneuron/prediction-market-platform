import { Button, Dialog, Field, Input } from "@chakra-ui/react";
import { useForm } from "react-hook-form";
import { useMutation } from "@tanstack/react-query";

import Modal from ".";
import { toaster } from "../ui/toaster";
import { MarketActions } from "@/utils/interactions/dataPosters";
import useRevalidation from "@/hooks/useRevalidate";
import useModal from "@/hooks/useModal";

type Props = {
  orderId: string;
  quantity: string;
  filledQuantity: string;
  price: string;
};

type FormSchema = {
  quantity: string;
  price: string;
};

const UpdateOrderModal = ({
  orderId,
  filledQuantity,
  quantity,
  price,
}: Props) => {
  const { register, handleSubmit } = useForm<FormSchema>({
    defaultValues: {
      quantity,
      price,
    },
  });
  const { mutateAsync, isPending } = useMutation({
    mutationFn: MarketActions.updateOrder,
  });
  const revalidate = useRevalidation();
  const { close } = useModal();

  function onSubmit(data: FormSchema) {
    if (data.quantity === quantity && data.price === price) {
      toaster.info({
        title: "No changes made",
        description: "You have not changed the quantity.",
        closable: true,
      });
      return;
    }
    if (Number(data.quantity) <= Number(filledQuantity)) {
      toaster.error({
        title: "Invalid Quantity",
        description:
          "Quantity cannot be less than or equal to filled quantity.",
        closable: true,
      });
      return;
    }
    // price must be between 0 and 1
    if (Number(data.price) < 0 || Number(data.price) > 1) {
      toaster.error({
        title: "Invalid Price",
        description: "Price must be between 0 and 1.",
        closable: true,
      });
      return;
    }
    toaster.promise(
      mutateAsync({
        order_id: orderId,
        new_quantity: Number(data.quantity),
        new_price: Number(data.price),
      }),
      {
        loading: { title: "Updating order..." },
        success: () => {
          revalidate(["marketOrders"]);
          close();
          return { title: "Order update request submitted successfully!" };
        },
        error: (error) => ({
          title: "Error updating order",
          description: error instanceof Error ? error.message : "Unknown error",
          closable: true,
        }),
      },
    );
  }

  return (
    <Modal type={`update-order-${orderId}`} closeOnInteractOutside={false}>
      <Dialog.Header>
        <Dialog.Title>Update Order</Dialog.Title>
      </Dialog.Header>
      <form onSubmit={handleSubmit(onSubmit)}>
        <Dialog.Body spaceY={4}>
          <Field.Root>
            <Field.Label>Quantity (Filled: {filledQuantity})</Field.Label>
            <Input
              placeholder="10"
              {...register("quantity", {
                required: {
                  message: "Quantity is required",
                  value: true,
                },
              })}
              disabled={isPending}
              type="number"
              step={1}
            />
          </Field.Root>
          <Field.Root>
            <Field.Label>Price (Current: {price})</Field.Label>
            <Input
              placeholder="0.5"
              {...register("price", {
                required: {
                  message: "Price is required",
                  value: true,
                },
              })}
              disabled={isPending}
              type="number"
              step={0.001}
            />
          </Field.Root>
        </Dialog.Body>
        <Dialog.Footer>
          <Button
            type="submit"
            colorScheme="blue"
            mt={4}
            disabled={isPending}
            loading={isPending}
          >
            Update Order
          </Button>
        </Dialog.Footer>
      </form>
    </Modal>
  );
};

export default UpdateOrderModal;
