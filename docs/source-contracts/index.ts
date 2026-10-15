export type Edit = {
  adjustmentOpacity: number;
  adjustmentEnabled: boolean;
  exposure: number;
  contrast: number;
  saturation: number;
  temperature: number;
  rotation: number;
  flipH: boolean;
  flipV: boolean;
  crop: number[] | null;
};
export type Asset = {
  id: string;
  name: string;
  width: number;
  height: number;
  bytes: number;
  trashed: boolean;
  edit: Edit;
  versions: { id: string; edit: Edit; createdAt: number; kind: string }[];
  generatedBase?: string | null;
  exported: boolean;
  sourcePath: string;
};
export type Project = {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
  assets: Asset[];
};
export type Preset = {
  format: string;
  width: number;
  height: number;
  fit: string;
  quality: number;
  maxBytes: number;
  outputDir: string;
  background: number[];
};
