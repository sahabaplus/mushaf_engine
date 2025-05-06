import { LINES_PER_PAGE, MAX_PAGE, MIN_PAGE } from "../constants";
import type {
  LinesBetweenPagesProps,
  LinesBetweenSurasProps,
  NavigateFromVerseProps,
  OnBeforeLastVerse,
  OnOverflowStrategyFn,
} from "../i_quran_engine_base";
import type { Direction, Verse, SuraNumber } from "../types";
import { BaseQuranEngine } from "./base_quran_engine";

export class BackwardDirectionBaseEngine extends BaseQuranEngine {
  public linesBetweenPages(p: LinesBetweenPagesProps): number {
    const { direction, endPage, startPage } = p;
    if (direction !== -1)
      throw new Error(
        "BackwardDirectionBaseEngine cannot calculate for forward"
      );

    if (endPage > MAX_PAGE || startPage < MIN_PAGE)
      throw new Error(`Invalid Range ${endPage + " - " + startPage}`);

    const lines = LINES_PER_PAGE * (startPage - endPage + 1);

    // In backward direction, we need to handle special cases for pages 1 and 2
    const specialCaseDeduction = endPage in [1, 2] ? endPage : 0;
    return lines - specialCaseDeduction * 15;
  }

  public linesBetweenSuras(p: LinesBetweenSurasProps): number {
    const { direction, endSura, startSura } = p;
    if (direction !== -1)
      throw new Error(
        "BackwardDirectionBaseEngine cannot calculate for forward"
      );

    const { end_page } = this.surasSizesInfosMap.get(startSura)!;
    const { start_page } = this.surasSizesInfosMap.get(endSura)!;

    return (end_page - start_page + 1) * LINES_PER_PAGE;
  }

  public navigate({
    direction,
    fromLine,
    fromPage,
    lines,
  }: {
    lines: number;
    direction: Direction;
    fromPage: number;
    fromLine: number;
  }): { verse: Verse; page: number } {
    if (direction !== -1)
      throw new Error(
        "BackwardDirectionBaseEngine cannot calculate for forward"
      );

    const startPage = fromPage;

    const startSura = this.getSuraFromPage({
      page: startPage,
      line: fromLine,
    });

    const startVerse = this.getVerseFromLocation({
      line: fromLine,
      page: startPage,
      onOverflowStrategy: ({ overflowBy, overflowVerse, previousVerse }) => {
        if (overflowBy / lines >= 0.2 && previousVerse) return previousVerse;
        return overflowVerse;
      },
      onBeforeLastVerse: ({ currentVerse, lastVerse, overflowBy }) => {
        if (overflowBy / lines >= 0.2) return currentVerse;
        return lastVerse;
      },
    });

    const widthUntilEnd = this.calculateSuraLines({
      sura: startSura,
      startVerse: startVerse.ayah,
    });

    let remainingLines = lines + fromLine;
    let endPage = startPage;
    let currentSura = startSura;

    while (remainingLines > 0) {
      const {
        end_page,
        start_page,
        lines: currentSuraLines,
      } = this.surasSizesInfosMap.get(currentSura)!;

      const suraWidth =
        startSura === currentSura ? widthUntilEnd : currentSuraLines;

      // console.log(`Trying to see whither sura:${currentSura} fits or no!`, {
      //   currentSura,
      //   suraWidth,
      //   remainingLines,
      // });

      if (remainingLines - suraWidth <= 0) {
        endPage =
          Math.floor(remainingLines / 15) +
          (startSura === currentSura ? fromPage : start_page);
        remainingLines %= LINES_PER_PAGE;

        break;
      }

      remainingLines -= suraWidth;
      currentSura -= 1;
    }
    const endVerse = this.getVerseFromLocation({
      line: remainingLines,
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
    const result = this.approximateToNearestVerse({
      // startLine:,
      lines: remainingLines,
      onBeforeLastVerse: ({ currentVerse }) => currentVerse,
      page: endPage,
    });

    return {
      page: endPage,
      verse: result,
    };
    throw new Error("unimplemented");
    const verse = this.approximateToNearestVerse({
      lines: remainingLines,
      page: endPage,
      onOverflowStrategy: ({ overflowBy, overflowVerse, previousVerse }) => {
        if (overflowBy / lines >= 0.2 && previousVerse) return previousVerse;
        return overflowVerse;
      },
      onBeforeLastVerse: ({ currentVerse, lastVerse, overflowBy }) => {
        if (overflowBy / lines >= 0.2) return lastVerse;
        return currentVerse;
      },
    });

    if (!verse) throw new Error("Could not approximateToNearestVerse");

    return {
      page: endPage,
      verse,
    };
  }

  private getSuraFromPage({
    page,
    line,
  }: {
    page: number;
    line: number;
  }): SuraNumber {
    const { verses } = this.pages.get(page)!;
    for (const verse of verses) {
      if (line - verse.y <= 0) return verse.sura;
    }

    throw new Error(
      `getSuraFromPage could not find at ${JSON.stringify({
        page,
        line,
      })}`
    );
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
      const currentSuraWidth =
        this.calculateSuraLines({
          sura: currentSura,
          startVerse:
            currentSura === startVerse.sura ? startVerse.ayah : undefined,
        }) + 2; // Header

      if (+currentSuraWidth.toFixed() >= +remainingLines.toFixed()) {
        // Found the end sura!
        break;
      }

      // console.log("Moving Backwards from", {
      //   currentSura,
      //   remainingLines,
      //   d: [currentSuraWidth.toFixed(), remainingLines.toFixed()],
      //   boo: +currentSuraWidth.toFixed() >= +remainingLines.toFixed(),
      // });
      remainingLines -= currentSuraWidth;
      currentSura -= 1; // Backward movement
      // console.log("To", {
      //   currentSura,
      //   remainingLines,
      // });
    }

    const verse = this.navigateInSura({
      byLines: +(remainingLines - 2).toFixed(),
      sura: currentSura,
      onBeforeLastVerse,
      onOverflowStrategy,
    });

    return verse;
  }
}
