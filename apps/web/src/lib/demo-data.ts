export interface DemoTemplate {
  id: string;
  label: string;
  type: string;
  previewSrc: string;
}

export interface DemoDesign {
  id: string;
  label: string;
  previewSrc: string;
  publicPath: string;
}

export const demoTemplates: DemoTemplate[] = [
  {
    id: "white_male_front",
    label: "T-Shirt",
    type: "t-shirt",
    previewSrc: "/templates/template-tshirt.png",
  },
  {
    id: "hoodie-aop-front-132947",
    label: "Hoodie",
    type: "hoodie",
    previewSrc: "/templates/template-hoodie.png",
  },
  {
    id: "mug-11oz-front-919",
    label: "Mug",
    type: "mug",
    previewSrc: "/templates/template-mug.png",
  },
  {
    id: "phone-case-front-146439",
    label: "Phone Case",
    type: "phone-case",
    previewSrc: "/templates/template-phone-case.png",
  },
  {
    id: "pillow-front-22665",
    label: "Pillow",
    type: "pillow",
    previewSrc: "/templates/template-pillow.png",
  },
  {
    id: "poster-front-21372",
    label: "Poster",
    type: "poster",
    previewSrc: "/templates/template-poster.png",
  },
];

export const demoDesigns: DemoDesign[] = [
  {
    id: "sunburst",
    label: "Sunburst",
    previewSrc: "/samples/sample-design-1.png",
    publicPath: "/samples/sample-design-1.png",
  },
  {
    id: "gridwave",
    label: "Gridwave",
    previewSrc: "/samples/sample-design-2.png",
    publicPath: "/samples/sample-design-2.png",
  },
  {
    id: "flux",
    label: "Flux Mono",
    previewSrc: "/samples/sample-design-3.png",
    publicPath: "/samples/sample-design-3.png",
  },
];

export const demoLimit = 5;
