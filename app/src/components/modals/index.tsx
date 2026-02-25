"use client";

import { CloseButton, Dialog, Portal } from "@chakra-ui/react";

import useModal, { ModalType } from "@/hooks/useModal";

type Props = {
  children: React.ReactNode;
  type: ModalType;
  closeOnInteractOutside?: boolean;
  scrollBehavior?: "inside" | "outside";
};

const Modal = ({
  children,
  type,
  closeOnInteractOutside = true,
  scrollBehavior = "outside",
}: Props) => {
  const { isOpen, type: modalType, close } = useModal();
  const isOpenModal = isOpen && modalType === type;
  return (
    <Dialog.Root
      open={isOpenModal}
      onOpenChange={close}
      closeOnInteractOutside={closeOnInteractOutside}
      scrollBehavior={scrollBehavior}
    >
      <Portal>
        <Dialog.Backdrop
          backgroundColor={"rgba(0, 0, 0, 0.6)"}
          backdropBlur={"lg"}
        />
        <Dialog.Positioner>
          <Dialog.Content>
            {children}
            <Dialog.CloseTrigger asChild>
              <CloseButton size="sm" />
            </Dialog.CloseTrigger>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  );
};

export default Modal;
