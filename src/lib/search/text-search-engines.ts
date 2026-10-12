// Text search engines. Port of CTS/Search/GoogleSearchUrl.cs + TextSearchUrlBuilder.cs.
// All are pure URL builders — no platform coupling.

export interface TextSearchEngine {
  id: string;
  name: string;
  buildUrl(query: string): string;
}

function encoded(query: string): string {
  return encodeURIComponent(query);
}

export const textSearchEngines: TextSearchEngine[] = [
  {
    id: "google",
    name: "Google",
    buildUrl: (q) => `https://www.google.com/search?q=${encoded(q)}`,
  },
  {
    id: "bing",
    name: "Bing",
    buildUrl: (q) => `https://www.bing.com/search?q=${encoded(q)}`,
  },
  {
    id: "duckduckgo",
    name: "DuckDuckGo",
    buildUrl: (q) => `https://duckduckgo.com/?q=${encoded(q)}`,
  },
  {
    id: "kagi",
    name: "Kagi",
    buildUrl: (q) => `https://kagi.com/search?q=${encoded(q)}`,
  },
  {
    id: "qwant",
    name: "Qwant",
    buildUrl: (q) => `https://www.qwant.com/?q=${encoded(q)}`,
  },
  {
    id: "startpage",
    name: "Startpage",
    buildUrl: (q) => `https://www.startpage.com/sp/search?query=${encoded(q)}`,
  },
];

// "Match image search": plain Google query, matching CircleFlow's default behavior
// where the text search engine mirrors the visual provider when applicable.
export const matchImageSearchEngine: TextSearchEngine = {
  id: "matchimagesearch",
  name: "Match image search",
  buildUrl: (q) => `https://www.google.com/search?q=${encoded(q)}`,
};

export function engineById(id: string): TextSearchEngine {
  return textSearchEngines.find((e) => e.id === id) ?? matchImageSearchEngine;
}
