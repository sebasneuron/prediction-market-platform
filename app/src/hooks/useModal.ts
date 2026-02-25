import { create } from "zustand";

export type ModalType = `update-order-${string}`;
interface ModelState {
  type: ModalType | null;
  isOpen: boolean;
  open: (type: ModalType) => void;
  close: () => void;
}

const useModal = create<ModelState>((set) => ({
  type: null,
  isOpen: false,
  open: (type: ModalType) => set({ type, isOpen: true }),
  close: () => set({ type: null, isOpen: false }),
}));

export default useModal;
