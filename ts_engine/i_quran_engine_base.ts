import type { Direction, SuraNumber, Verse, VersePosition } from "./types";

export type LinesBetweenPagesProps = {
  startPage: number;
  endPage: number;
  direction: Direction;
};
export type LinesBetweenSurasProps = {
  startSura: SuraNumber;
  endSura: SuraNumber;
  direction: Direction;
};

export type GetVerseFromLocationProps = {
  line: number;
  page: number;
  onOverflowStrategy?: OnOverflowStrategyFn;
  onBeforeLastVerse?: OnBeforeLastVerse;
};

export type OnOverflowStrategyFn = (p: {
  previousVerse: VersePosition;
  overflowVerse: VersePosition;
  overflowBy: number;
}) => VersePosition;

export type OnBeforeLastVerse = (p: {
  currentVerse: VersePosition;
  lastVerse: VersePosition;
  overflowBy: number;
}) => VersePosition;

export type NavigateInSuraProps = {
  byLines: number;
  sura: number;
  startVerse?: number;
  onOverflowStrategy?: OnOverflowStrategyFn;
  onBeforeLastVerse?: OnBeforeLastVerse;
};

export type GetNextVerseProps = {
  verse: Verse;
  direction: Direction;
};

export type NavigateFromVerseProps = {
  startVerse: Verse;
  byLines: number;
  onOverflowStrategy?: OnOverflowStrategyFn;
  onBeforeLastVerse?: OnBeforeLastVerse;
  direction: Direction;
};

export abstract class IQuranEnginBase {
  public abstract linesBetweenPages(p: LinesBetweenPagesProps): number;

  public abstract linesBetweenSuras(p: LinesBetweenSurasProps): number;

  public abstract getPageVerses(page: number): VersePosition[];

  public abstract verseIsLastInPage(verse: VersePosition): boolean;

  public abstract navigate(p: {
    lines: number;
    direction: Direction;
    fromPage: number;
    fromLine: number;
  }): {
    verse: Verse;
    page: number;
  };

  public abstract approximateToNearestVerse(p: {
    page: number;
    lines: number;
    startLine?: number;
    onOverflowStrategy?: OnOverflowStrategyFn;
    onBeforeLastVerse: OnBeforeLastVerse;
  }): VersePosition;

  public abstract calculateSuraLines({
    startVerse,
    endVerse,
    sura,
  }: {
    startVerse?: number;
    endVerse?: number;
    sura: number;
  }): number;

  public abstract getVerseFromLocation({
    line,
    page,
    onBeforeLastVerse,
    onOverflowStrategy,
  }: GetVerseFromLocationProps): VersePosition;

  public abstract navigateFromVerse(p: NavigateFromVerseProps): Verse;

  public abstract getVerseLocation(verse: Verse): {
    page: number;
    verse: VersePosition;
  };
  public abstract getVerseHeight(verse: Verse): VersePosition;

  public abstract getNextVerse({ verse, direction }: GetNextVerseProps): Verse;

  public abstract navigateInSura(p: NavigateInSuraProps): Verse;
}
