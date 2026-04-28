declare module "diff" {
  interface DiffPart {
    value: string;
    added?: boolean;
    removed?: boolean;
  }

  export function diffLines(
    oldStr: string,
    newStr: string,
    options?: { ignoreWhitespace?: boolean; newlineIsToken?: boolean },
  ): DiffPart[];

  export function diffChars(
    oldStr: string,
    newStr: string,
    options?: object,
  ): DiffPart[];

  export function diffWords(
    oldStr: string,
    newStr: string,
    options?: object,
  ): DiffPart[];
}
