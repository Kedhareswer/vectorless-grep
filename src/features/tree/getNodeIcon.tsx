import { TextIcon, TableIcon, FigureIcon, DocumentIcon, SectionIcon } from "./TreeIcons";

export function getNodeIcon(nodeType: string) {
  const type = nodeType.toLowerCase();
  if (type === "table") return <TableIcon />;
  if (type === "figure" || type === "image") return <FigureIcon />;
  if (type === "document") return <DocumentIcon />;
  if (type === "section" || type === "subsection") return <SectionIcon />;
  return <TextIcon />;
}
