import { TERM_HELP } from "../constants";
import { TermHelp } from "./TermHelp";
export function HelpHeading({ children, term }: { children: string; term: keyof typeof TERM_HELP }) {
  return <span className="helpHeading"><span>{children}</span><TermHelp term={term} /></span>;
}
