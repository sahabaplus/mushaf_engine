export type SuraNumber = number;
export type PageNumber = number;

export type SuraSizeInfo = {
  start_page: number;
  end_page: number;
  lines: number;
  totalVerses: number;
};
export interface VersePosition {
  sura: SuraNumber;
  ayah: number;
  y: number;
  x: number;
  height: number;
}

export type Verse = {
  sura: SuraNumber;
  ayah: number;
};
export type PageData = { page: number; verses: VersePosition[] };

export interface PlanProgress {
  currentSura: SuraNumber;
  currentAya: number;
}

export type Direction = 1 | -1;

export interface BasePlan {
  type: "static" | "dynamic";
  direction: Direction;
}

export interface StaticPlanConfig extends BasePlan {
  type: "static";
  amount: number;
  progress: PlanProgress;
}

export interface DynamicPlanConfig extends BasePlan {
  type: "dynamic";
  endSura: SuraNumber;
  endAya: number;
  remainingRecitations: number;
  progress: PlanProgress;
}
