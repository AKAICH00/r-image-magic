export interface MockupColor {
  id: string;
  name: string;
  hex: string;
}

export const CORE_COLORS: MockupColor[] = [
  { id: "white", name: "White", hex: "FFFFFF" },
  { id: "black", name: "Black", hex: "0D0D0D" },
  { id: "navy", name: "Navy", hex: "0F1F3D" },
  { id: "heather_gray", name: "Athletic Heather", hex: "9EA1A2" },
  { id: "red", name: "Red", hex: "C8102E" },
  { id: "royal", name: "True Royal", hex: "234B9D" },
];

export const TINTABLE_PRODUCT_TYPES = ["t-shirt", "hoodie", "tank"];
