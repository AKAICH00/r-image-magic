export interface TemplateCategory {
  productType: string;
  label: string;
  count: number;
  thumbnailSrc: string;
  templateId: string;
}

export const templateCategories: TemplateCategory[] = [
  {
    productType: "t-shirt",
    label: "T-Shirts",
    count: 7,
    thumbnailSrc: "/templates/template-tshirt.png",
    templateId: "white_male_front",
  },
  {
    productType: "hoodie",
    label: "Hoodies",
    count: 6,
    thumbnailSrc: "/templates/template-hoodie.png",
    templateId: "hoodie-aop-front-132947",
  },
  {
    productType: "mug",
    label: "Mugs",
    count: 5,
    thumbnailSrc: "/templates/template-mug.png",
    templateId: "mug-front-919",
  },
  {
    productType: "phone-case",
    label: "Phone Cases",
    count: 6,
    thumbnailSrc: "/templates/template-phone-case.png",
    templateId: "phone-case-front-146439",
  },
  {
    productType: "pillow",
    label: "Pillows",
    count: 6,
    thumbnailSrc: "/templates/template-pillow.png",
    templateId: "pillow-front-22665",
  },
  {
    productType: "poster",
    label: "Posters",
    count: 6,
    thumbnailSrc: "/templates/template-poster.png",
    templateId: "poster-front-21372",
  },
  {
    productType: "tote",
    label: "Totes",
    count: 3,
    thumbnailSrc: "/templates/template-tote.png",
    templateId: "tote-front-1204",
  },
  {
    productType: "wrapping-paper",
    label: "Wrapping Paper",
    count: 3,
    thumbnailSrc: "/templates/template-wrapping-paper.png",
    templateId: "wrapping-paper-front-196986",
  },
  {
    productType: "tank",
    label: "Tanks",
    count: 2,
    thumbnailSrc: "/templates/template-tank.png",
    templateId: "tank-aop-front-4245",
  },
];

export const totalTemplates = templateCategories.reduce(
  (sum, category) => sum + category.count,
  0,
);
