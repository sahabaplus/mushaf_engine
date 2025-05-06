import type { StaticPlanConfig, Verse } from "../../types";
import { PlanQuranEngine } from "./plan_quran_engine";

export class StaticPlanEngine extends PlanQuranEngine {
  public calculateNextRecitation(p: { plan: StaticPlanConfig }): Verse {
    const {
      amount,
      direction,
      progress: { currentAya, currentSura },
    } = p.plan;

    // console.log(
    //   {
    //     byLines: amount,
    //     direction,
    //     startVerse: {
    //       sura: currentSura,
    //       ayah: currentAya,
    //     },
    //   },
    //   {
    //     byLines: amount,
    //     direction,
    //     startVerse: {
    //       sura: currentSura,
    //       ayah: currentAya,
    //     },
    //   },
    // );
    return this.navigateFromVerse({
      byLines: amount,
      direction,
      startVerse: {
        sura: currentSura,
        ayah: currentAya,
      },
      onOverflowStrategy: ({ overflowBy, overflowVerse, previousVerse }) => {
        if (overflowBy < 0.8) {
          return previousVerse;
        }
        if (
          overflowBy / (amount - overflowBy) >= 0.25 &&
          overflowVerse.ayah !== currentAya
        )
          return previousVerse;
        return overflowVerse;
      },
      onBeforeLastVerse: ({ currentVerse, lastVerse, overflowBy }) => {
        if (overflowBy / amount >= 0.2) return currentVerse;
        return lastVerse;
      },
    });
  }
}
