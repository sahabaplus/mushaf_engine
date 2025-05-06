import { LINES_PER_PAGE, MAX_PAGE, MIN_PAGE } from "../constants";
import type {
  GetNextVerseProps,
  LinesBetweenPagesProps,
  LinesBetweenSurasProps,
  NavigateFromVerseProps,
} from "../i_quran_engine_base";
import type { Direction, Verse } from "../types";
import { BaseQuranEngine } from "./base_quran_engine";

export class ForwardDirectionBaseEngine extends BaseQuranEngine {
  public linesBetweenPages(p: LinesBetweenPagesProps): number {
    const { direction, endPage, startPage } = p;
    if (direction !== 1)
      throw new Error(
        "ForwardDirectionBaseEngine cannot calculate for backward"
      );

    if (endPage > MAX_PAGE || startPage < MIN_PAGE)
      throw new Error(`Invalid Rage ${endPage + " - " + startPage}`);

    const lines = LINES_PER_PAGE * (endPage - startPage + 1);

    const specialCaseDeduction = startPage in [1, 2] ? startPage : 0;
    return lines - specialCaseDeduction * 15;
  }
  public linesBetweenSuras(p: LinesBetweenSurasProps): number {
    const { direction, endSura, startSura } = p;
    if (direction !== 1)
      throw new Error(
        "ForwardDirectionBaseEngine cannot calculate for backward"
      );

    const { start_page } = this.surasSizesInfosMap.get(startSura)!;
    const { end_page } = this.surasSizesInfosMap.get(endSura)!;

    return (start_page - end_page + 1) * LINES_PER_PAGE;
  }

  public navigate({
    direction,
    lines,
    fromPage,
    fromLine,
  }: {
    lines: number;
    direction: Direction;
    fromPage: number;
    fromLine: number;
  }): { verse: Verse; page: number } {
    if (direction !== 1)
      throw new Error(
        "ForwardDirectionBaseEngine cannot calculate for backward"
      );
    const totalPages = Math.floor((lines + (fromLine - 1)) / LINES_PER_PAGE);
    // + (this.verseIsLastInPage(fromVerse) ? 1 : fromVerse.y + fromVerse.x);

    const endPage =
      totalPages + fromPage + (fromPage in [1, 2] ? 2 - fromPage : 0);

    const remainingLines = (lines + (fromLine - 1)) % LINES_PER_PAGE;

    const verse = this.approximateToNearestVerse({
      lines: remainingLines,
      page: endPage,
      onOverflowStrategy: ({ overflowBy, overflowVerse, previousVerse }) => {
        if (overflowBy / lines >= 0.2 && previousVerse) return previousVerse;
        return overflowVerse;
      },
      onBeforeLastVerse: ({ currentVerse, lastVerse, overflowBy }) => {
        if (overflowBy / lines >= 0.2) return currentVerse;
        return lastVerse;
      },
    });

    if (!verse) throw new Error("Could not approximateToNearestVerse");

    return {
      page: endPage,
      verse,
    };
  }

  public _navigateFromVerse({
    byLines,
    startVerse,
    onBeforeLastVerse,
    onOverflowStrategy,
  }: NavigateFromVerseProps): Verse {
    let remainingLines = byLines;
    let currentSura = startVerse.sura;
    while (remainingLines > 0 && currentSura > 0 && currentSura <= 114) {
      const currentSuraWidth = this.calculateSuraLines({
        sura: currentSura,
        startVerse:
          currentSura === startVerse.sura ? startVerse.ayah : undefined,
      });

      if (currentSuraWidth - remainingLines > 0) {
        // Found the end sura!
        // remainingLines +=
        //   this.calculateSuraLines({ sura: currentSura }) - currentSuraWidth;
        break;
      }

      remainingLines -= currentSuraWidth;

      currentSura += 1; // Forward movement
      if (remainingLines > 0 && currentSura > 114) {
        currentSura = 114;
        remainingLines = currentSuraWidth;
        break;
      }
    }

    const verse = this.navigateInSura({
      byLines: remainingLines,
      sura: currentSura,
      startVerse: startVerse.ayah,
      onBeforeLastVerse,
      onOverflowStrategy,
    });

    return verse;
  }
}
