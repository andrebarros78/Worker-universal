import type { AriaRole, Locator, Page } from "playwright";

export type SemanticTarget = {
  role?: AriaRole;
  name?: string;
  label?: string;
  placeholder?: string;
  testId?: string;
  text?: string;
  css?: string;
};

export type ResolvedLocator = {
  locator: Locator;
  strategy: string;
};

type Candidate = {
  strategy: string;
  locator: Locator;
};

export async function resolveLocator(
  page: Page,
  target: SemanticTarget,
): Promise<ResolvedLocator> {
  const candidates: Candidate[] = [];

  if (target.role) {
    candidates.push({
      strategy: "role",
      locator: page.getByRole(target.role, {
        name: target.name,
        exact: target.name !== undefined,
      }),
    });
  }
  if (target.label) {
    candidates.push({
      strategy: "label",
      locator: page.getByLabel(target.label, { exact: true }),
    });
  }
  if (target.placeholder) {
    candidates.push({
      strategy: "placeholder",
      locator: page.getByPlaceholder(target.placeholder, { exact: true }),
    });
  }
  if (target.testId) {
    candidates.push({
      strategy: "testid",
      locator: page.getByTestId(target.testId),
    });
  }
  if (target.text) {
    candidates.push({
      strategy: "text",
      locator: page.getByText(target.text, { exact: true }),
    });
  }
  if (target.css) {
    candidates.push({
      strategy: "css",
      locator: page.locator(target.css),
    });
  }

  if (candidates.length === 0) {
    throw new Error("semantic_target_empty");
  }

  for (const candidate of candidates) {
    const count = await candidate.locator.count();
    if (count > 0) {
      return { locator: candidate.locator.first(), strategy: candidate.strategy };
    }
  }

  throw new Error("element_not_found:" + JSON.stringify(target));
}
