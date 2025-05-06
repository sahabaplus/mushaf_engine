import type { BasePlan, Verse } from "../../types";
import { NavigationQuranEngine } from "../navigation_quran_engine";

export type CalculateNextProps = { plan: BasePlan };
export abstract class PlanQuranEngine extends NavigationQuranEngine {
  public abstract calculateNextRecitation({ plan }: CalculateNextProps): Verse;
}
