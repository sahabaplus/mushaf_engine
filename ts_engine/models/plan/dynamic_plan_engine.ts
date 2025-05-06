import type { DynamicPlanConfig, Verse } from "../../types";
import { PlanQuranEngine } from "./plan_quran_engine";

export class DynamicPlanEngine extends PlanQuranEngine {
  public calculateNextRecitation({ plan }: { plan: DynamicPlanConfig }): Verse {
    const { direction, endAya, endSura, progress, remainingRecitations } = plan;

    if (progress.currentAya >= endAya && progress.currentSura === endSura)
      throw new Error("Plan has ended!");

    if (remainingRecitations <= 0) throw new Error("remainingRecitations <= 0");

    if (direction === 1 && endSura < progress.currentSura)
      throw new Error("endSura > progress.currentSura");

    if (direction === -1 && endSura > progress.currentSura)
      throw new Error("endSura > progress.currentSura");

    if (remainingRecitations === 1)
      return {
        ayah: endAya,
        sura: endSura,
      };

    // const start_sura = direction === 1 ? progress.currentSura : endSura;
    // const end_sura = direction === 1 ? endSura : progress.currentSura;

    let totalLines = 0;
    for (let currentSura = progress.currentSura; ; currentSura += direction) {
      const lines = this.calculateSuraLines({
        sura: currentSura,
        startVerse:
          currentSura === progress.currentSura
            ? progress.currentAya
            : undefined,
        endVerse: currentSura === endSura ? endAya : undefined,
      });
      //+ (!(currentSura === start_sura ? progress.currentAya : undefined) ? 0 : 0);

      totalLines += lines;
      // if (start_sura === end_sura) break;

      if (currentSura === endSura) break;
    }

    const todayAmount = Math.ceil(totalLines / remainingRecitations);

    console.log({
      todayAmount,
      totalLines,
    });
    if (todayAmount === 0)
      throw new Error(
        "Either the plan has finished, or the amount is very small"
      );

    const result = this.navigateFromVerse({
      byLines: todayAmount,
      direction,
      startVerse: {
        sura: progress.currentSura,
        ayah: progress.currentAya,
      },
      onOverflowStrategy: ({ overflowBy, overflowVerse, previousVerse }) => {
        console.log("OVERFLOW", { overflowBy, overflowVerse, previousVerse });
        if (overflowVerse.y === previousVerse.y) return overflowVerse;

        if (
          1 - remainingRecitations <= 0 &&
          overflowVerse.ayah >= endAya &&
          overflowVerse.sura === overflowVerse.sura
        )
          return overflowVerse;

        if (overflowBy < 0.8 && overflowBy > 0.2) return previousVerse;

        if (
          overflowBy / (todayAmount - overflowBy) >= 0.4 &&
          overflowVerse.ayah !== progress.currentAya
        )
          return previousVerse;
        return overflowVerse;
      },
      onBeforeLastVerse: ({ currentVerse, lastVerse, overflowBy }) => {
        if (lastVerse.ayah === endAya) return lastVerse;
        if (overflowBy / todayAmount >= 0.2) return currentVerse;
        return lastVerse;
      },
    });

    return {
      ayah: result.ayah,
      sura: result.sura,
    };
  }
}
