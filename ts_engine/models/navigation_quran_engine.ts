import type {
  LinesBetweenPagesProps,
  LinesBetweenSurasProps,
  NavigateFromVerseProps,
} from "../i_quran_engine_base";
import type { Direction, Verse, VersePosition } from "../types";
import { BackwardDirectionBaseEngine } from "./backward_direction_base_engine";
import { BaseQuranEngine } from "./base_quran_engine";
import { ForwardDirectionBaseEngine } from "./forward_direction_base_engine";

export class NavigationQuranEngine extends BaseQuranEngine {
  private getEngine(
    direction: Direction
  ): ForwardDirectionBaseEngine | BackwardDirectionBaseEngine {
    return direction === 1
      ? new ForwardDirectionBaseEngine({
          quranPagesMap: this.pages,
          surasSizesInfosMap: this.surasSizesInfosMap,
        })
      : new BackwardDirectionBaseEngine({
          quranPagesMap: this.pages,
          surasSizesInfosMap: this.surasSizesInfosMap,
        });
  }

  public navigate(p: {
    lines: number;
    direction: Direction;
    fromPage: number;
    fromLine: number;
  }): { verse: Verse; page: number } {
    const engine = this.getEngine(p.direction);
    return engine.navigate(p);
  }

  public navigateFromVerse(p: NavigateFromVerseProps): Verse {
    const engine = this.getEngine(p.direction);
    return engine.navigateFromVerse({
      ...p,
    });
  }

  public linesBetweenPages(p: LinesBetweenPagesProps): number {
    const engine = this.getEngine(p.direction);

    return engine.linesBetweenPages(p);
  }
  public linesBetweenSuras(p: LinesBetweenSurasProps): number {
    const engine = this.getEngine(p.direction);

    return engine.linesBetweenSuras(p);
  }
  public getVerseLocationInPage(
    sura: number,
    aya: number
  ): { verse: VersePosition; page: number } {
    const range = this.surasSizesInfosMap.get(sura);
    if (!range) throw new Error(`Invalid sura number: ${sura}`);

    for (let page = range.start_page; page <= range.end_page; page++) {
      const verses = this.getPageVerses(page);
      const verse = verses.find((v) => v.sura === sura && v.ayah === aya);
      if (verse) return { verse, page };
    }
    throw new Error(`Verse not found: Sura ${sura}, Aya ${aya}`);
  }

  public getPreviousVerse(sura: number, aya: number): VersePosition {
    const currentPage = this.getVerseLocationInPage(sura, aya);
    const verses = this.getPageVerses(currentPage.page);

    const currentIndex = verses.findIndex(
      (v) => v.sura === sura && v.ayah === aya
    );

    if (currentIndex > 0) {
      return verses[currentIndex - 1];
    }

    // If at start of page, get last verse of previous page
    const prevPage = currentPage.page - 1;
    if (prevPage >= 1) {
      const prevVerses = this.getPageVerses(prevPage);
      return prevVerses[prevVerses.length - 1];
    }

    throw new Error("No previous verse available - start of Quran reached");
  }

  public getLinesBetweenVerses(
    start: { verse: VersePosition; page: number },
    end: { verse: VersePosition; page: number }
  ): number {
    if (start.page === end.page) {
      return Math.abs(end.verse.y - start.verse.y);
    }

    const direction = end.page > start.page ? 1 : -1;

    return this.linesBetweenPages({
      startPage: start.page,
      endPage: end.page,
      direction,
    });
  }

  public getVersePage(
    sura: number,
    aya: number
  ): { verses: VersePosition[]; pageNumber: number } {
    const range = this.surasSizesInfosMap.get(sura);
    if (!range) throw new Error(`Invalid sura number: ${sura}`);

    for (let page = range.start_page; page <= range.end_page; page++) {
      const verses = this.getPageVerses(page);
      if (verses.some((v) => v.sura === sura && v.ayah === aya)) {
        return { pageNumber: page, verses };
      }
    }
    throw new Error(`Verse not found: Sura ${sura}, Aya ${aya}`);
  }

  private getLastPage(): number {
    return Math.max(...Array.from(this.pages.keys()));
  }
}
