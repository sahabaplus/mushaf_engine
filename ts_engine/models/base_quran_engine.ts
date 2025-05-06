import { LINES_PER_PAGE } from "../constants";
import {
  IQuranEnginBase,
  type GetNextVerseProps,
  type GetVerseFromLocationProps,
  type NavigateFromVerseProps,
  type NavigateInSuraProps,
  type OnBeforeLastVerse,
  type OnOverflowStrategyFn,
} from "../i_quran_engine_base";
import type {
  PageData,
  PageNumber,
  SuraSizeInfo,
  SuraNumber,
  VersePosition,
  Verse,
} from "../types";
export type BaseQuranEngineProps = {
  quranPagesMap: Map<PageNumber, PageData>;
  surasSizesInfosMap: Map<SuraNumber, SuraSizeInfo>;
};
export abstract class BaseQuranEngine extends IQuranEnginBase {
  protected readonly pages: Map<PageNumber, PageData>;
  protected readonly surasSizesInfosMap: Map<SuraNumber, SuraSizeInfo>;
  constructor(p: BaseQuranEngineProps) {
    super();

    this.pages = p.quranPagesMap;
    this.surasSizesInfosMap = p.surasSizesInfosMap;
  }

  public getPageVerses(page: number): VersePosition[] {
    const result = this.pages.get(page);
    if (!result) throw new Error(`Could not find page verses for page:${page}`);

    return result.verses;
  }

  public verseIsLastInPage(verse: VersePosition): boolean {
    if (verse.sura in [1, 2]) {
      return (
        (verse.sura === 1 && verse.ayah === 7) ||
        (verse.sura === 2 && verse.ayah === 5)
      );
    }
    return verse.y === 15 && verse.x === 1;
  }

  public approximateToNearestVerse({
    lines,
    onBeforeLastVerse,
    page,
    onOverflowStrategy,
    startLine,
  }: {
    page: number;
    lines: number;
    startLine?: number;
    onOverflowStrategy?: OnOverflowStrategyFn;
    onBeforeLastVerse?: OnBeforeLastVerse;
  }): VersePosition {
    const { verses } = this.pages.get(page)!;
    const { verses: previousPageVerses } = this.pages.get(page - 1)!;

    let remainingLines = lines;
    let totalLines = 0;

    let preViousVerse: VersePosition =
      previousPageVerses[previousPageVerses.length - 1];
    for (let i = 0; i < verses.length; i++) {
      const verse = verses[i];

      if (startLine && verse.y < startLine) continue;

      if (i > 0) {
        preViousVerse = verses[i - 1]!;
      }

      totalLines += verse.height;

      const result = remainingLines - verse.height;

      if (result < 0) {
        const { totalVerses } = this.surasSizesInfosMap.get(verse.sura)!;
        const { verse: lastVerse } = this.getVerseLocation({
          ayah: totalVerses,
          sura: verse.sura,
        });
        const diff = this.calculateSuraLines({
          sura: verse.sura,
          startVerse: verse.ayah,
        });

        if (totalVerses - verse.ayah <= 2 && onBeforeLastVerse) {
          const returned = onBeforeLastVerse({
            currentVerse: verse,
            lastVerse: lastVerse,
            overflowBy: diff,
          });
          if (returned.ayah == lastVerse.ayah) return returned;
        }
      }

      if (
        i !== verses.length - 1 &&
        // LINES_PER_PAGE - totalLines <= LINES_PER_PAGE / 5 &&
        verse.y >= 13 &&
        verses[verses.length - 1].height > result &&
        onBeforeLastVerse
      ) {
        return onBeforeLastVerse({
          currentVerse: verse,
          lastVerse: verses[verses.length - 1],
          overflowBy: verses[verses.length - 1].height - result,
        });
      }

      if (result === 0) return verse;

      // if (result < 0 && verse === verses[verses.length - 1]) return verse;
      if (result < 0) {
        if (onOverflowStrategy)
          return onOverflowStrategy({
            overflowBy: -result,
            overflowVerse: verse,
            previousVerse: preViousVerse,
          });

        const reasonableOverflow = 1;

        if (-result > reasonableOverflow) return preViousVerse;

        return verse;
      }
      preViousVerse = verse;
      remainingLines = result;
    }
    if (preViousVerse) return preViousVerse;

    throw new Error("Could approximateToNearestVerse");
  }

  public calculateSuraLines({
    startVerse,
    endVerse,
    sura,
  }: {
    startVerse?: number;
    endVerse?: number;
    sura: number;
  }): number {
    const {
      end_page: sura_end_page,
      lines,
      start_page: sura_start_page,
      totalVerses,
    } = this.surasSizesInfosMap.get(sura)!;

    if (startVerse && startVerse > totalVerses)
      throw new Error(
        `Max number of verses for sura:${sura} is ${totalVerses}, given ${startVerse} as start`
      );
    if (
      (!startVerse || startVerse === 1) &&
      (!endVerse || endVerse === totalVerses)
    ) {
      // which means it needs the whole sura's size.
      // Hence, we will return the precalculated one.
      return lines;
    }

    const info = {
      startVerse: startVerse ?? 1,
      endVerse: endVerse ?? -1,
      startPage: sura_start_page,
      endPage: sura_end_page,
    };
    let totalCalculatedLines = 0;
    for (let page = sura_start_page; page <= sura_end_page; page++) {
      // const pagesVerses = this.calculatePageVersesHeight(page);
      const { verses: pagesVerses } = this.pages.get(page)!;
      const firstPageVerse = pagesVerses[0];
      const lastPageVerse = pagesVerses[pagesVerses.length - 1];

      if (
        firstPageVerse.sura === sura &&
        firstPageVerse.ayah > info.endVerse &&
        info.endVerse !== -1
      )
        break;

      // If the page is totally for the sura
      if (
        firstPageVerse.sura === lastPageVerse.sura &&
        firstPageVerse.ayah >= info.startVerse &&
        (lastPageVerse.ayah <= info.endVerse || info.endVerse == -1) &&
        !(page in [1, 2])
      ) {
        totalCalculatedLines += 15;
        continue;
      }

      // eslint-disable-next-line @typescript-eslint/prefer-for-of
      for (let i = 0; i < pagesVerses.length; i++) {
        const verse = pagesVerses[i];
        if (verse.sura !== sura) {
          continue;
        }

        if (
          verse.ayah >= info.startVerse &&
          (verse.ayah <= info.endVerse || info.endVerse == -1)
        ) {
          totalCalculatedLines += verse.height;
        }
      }
    }

    const total = +totalCalculatedLines.toFixed(2);
    if (total === 0) throw new Error("Out of bound!");

    return total;
  }

  public getVerseFromLocation({
    line,
    page,
  }: GetVerseFromLocationProps): VersePosition {
    const { verses } = this.pages.get(page)!;
    for (const verse of verses) {
      if (verse.y >= line) return verse;
    }

    throw new Error(
      `getSuraFromPage could not find at ${JSON.stringify({
        page,
        line,
      })}`
    );
  }

  public getVerseLocation(verse: Verse): {
    page: number;
    verse: VersePosition;
  } {
    const { end_page, start_page } = this.surasSizesInfosMap.get(verse.sura)!;

    for (let currentPage = start_page; currentPage <= end_page; currentPage++) {
      const { page, verses } = this.pages.get(currentPage)!;
      for (const currentVerse of verses) {
        if (currentVerse.sura !== verse.sura) continue;

        if (currentVerse.ayah === verse.ayah) {
          return {
            page,
            verse: currentVerse,
          };
        }
      }
    }

    throw new Error(
      `Could not find the verse location. ${JSON.stringify(verse)}`
    );
  }

  public navigateInSura({
    byLines,
    sura,
    startVerse,
    onBeforeLastVerse,
    onOverflowStrategy,
  }: NavigateInSuraProps): Verse {
    const startVerseNumber = startVerse ?? 1;
    const startVerseLocation = this.getVerseLocation({
      ayah: startVerseNumber,
      sura: sura,
    });
    const { lines } = this.surasSizesInfosMap.get(sura)!;
    const startVerseLine = Math.max(
      Math.floor(startVerseLocation.verse.y - startVerseLocation.verse.height) +
        1,
      0
    );

    // TODO: deduct the lines starting from the head of sura until the startVerse
    if (byLines > lines)
      throw new Error(
        `Trying to navigate in sura ${sura} with size of ${lines} lines by ${byLines} lines is not possible!`
      );

    const nextPage = Math.floor(
      (startVerseLine + byLines - 1) / (LINES_PER_PAGE + 1)
    );

    return this.approximateToNearestVerse({
      lines: nextPage
        ? byLines - (15 - startVerseLine)
        : byLines % (LINES_PER_PAGE + 1),
      startLine: nextPage ? undefined : startVerseLine % LINES_PER_PAGE,
      page: startVerseLocation.page + nextPage,
      // onOverflowStrategy,
      onBeforeLastVerse,
      onOverflowStrategy,
    });
  }

  public getVerseHeight(verse: Verse): VersePosition {
    return this.getVerseLocation({
      ayah: verse.ayah,
      sura: verse.sura,
    }).verse;
  }

  public getNextVerse({ verse, direction }: GetNextVerseProps): Verse {
    const currentSuraInfo = this.surasSizesInfosMap.get(verse.sura)!;

    if (verse.ayah < currentSuraInfo.totalVerses)
      return {
        ayah: verse.ayah + 1,
        sura: verse.sura,
      };

    return {
      ayah: 1,
      sura: Math.max((verse.sura + direction) % 115, 1),
    };
  }

  public navigateFromVerse({
    byLines,
    direction,
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

      currentSura += direction;
      if (remainingLines > 0 && currentSura > 114) {
        currentSura = 114;
        remainingLines = currentSuraWidth;
        break;
      }
    }

    const verse = this.navigateInSura({
      byLines: remainingLines,
      sura: currentSura,
      startVerse: startVerse.sura === currentSura ? startVerse.ayah : 1,
      onBeforeLastVerse,
      onOverflowStrategy,
    });

    return verse;
  }
}
