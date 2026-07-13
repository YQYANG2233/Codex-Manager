import type {
  ManagedModelImportPreviewV2Result,
  ManagedModelImportV2Params,
  ManagedModelListV2Result,
  ManagedModelV2,
  ManagedModelV2Upsert,
} from "@/types/model-v2";
import type { ModelInfo } from "@/types/model";

import { invoke, withAddr } from "./transport";
export {
  microusdToUsdPerMillion,
  usdPerMillionToMicrousd,
} from "./model-price-v2";

export const managedModelsV2Client = {
  list(includeHidden = false): Promise<ManagedModelListV2Result> {
    return invoke<ManagedModelListV2Result>(
      "service_managed_model_list_v2",
      withAddr({ includeHidden }),
    );
  },

  get(slug: string): Promise<ManagedModelV2> {
    return invoke<ManagedModelV2>(
      "service_managed_model_get_v2",
      withAddr({ slug }),
    );
  },

  upsert(input: ManagedModelV2Upsert): Promise<ManagedModelV2> {
    return invoke<ManagedModelV2>(
      "service_managed_model_upsert_v2",
      withAddr({ payload: input }),
    );
  },

  delete(slug: string): Promise<void> {
    return invoke<void>(
      "service_managed_model_delete_v2",
      withAddr({ slug }),
    );
  },

  previewImport(
    input: ManagedModelImportV2Params,
  ): Promise<ManagedModelImportPreviewV2Result> {
    return invoke<ManagedModelImportPreviewV2Result>(
      "service_managed_model_import_preview_v2",
      withAddr({ payload: input }),
    );
  },

  commitImport(
    input: ManagedModelImportV2Params,
  ): Promise<ManagedModelImportPreviewV2Result> {
    return invoke<ManagedModelImportPreviewV2Result>(
      "service_managed_model_import_commit_v2",
      withAddr({ payload: input }),
    );
  },
};

function capability(model: ManagedModelV2, ...keys: string[]): unknown {
  for (const key of keys) {
    if (key in model.capabilities) {
      return model.capabilities[key];
    }
  }
  return undefined;
}

function stringList(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value
    .map((item) => (typeof item === "string" ? item.trim() : ""))
    .filter(Boolean);
}

function booleanCapability(
  model: ManagedModelV2,
  fallback: boolean,
  ...keys: string[]
): boolean {
  const value = capability(model, ...keys);
  return typeof value === "boolean" ? value : fallback;
}

function nullableString(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value : null;
}

export function managedModelV2ToModelInfo(model: ManagedModelV2): ModelInfo {
  const reasoningEfforts = stringList(
    capability(model, "reasoningEfforts", "reasoning_efforts"),
  );
  const serviceTiers = stringList(
    capability(model, "serviceTiers", "service_tiers"),
  );
  return {
    slug: model.slug,
    displayName: model.displayName,
    description: model.description,
    defaultReasoningLevel: model.defaultReasoningEffort,
    supportedReasoningLevels: reasoningEfforts.map((effort) => ({
      effort,
      description: "",
    })),
    shellType: nullableString(capability(model, "shellType", "shell_type")),
    visibility: model.visibility,
    supportedInApi: model.supportedInApi,
    priority: model.sortOrder,
    additionalSpeedTiers: [],
    serviceTiers: serviceTiers.map((id) => ({ id, name: id, description: "" })),
    defaultServiceTier: nullableString(
      capability(model, "defaultServiceTier", "default_service_tier"),
    ),
    availabilityNux: null,
    upgrade: null,
    upgradeInfo: null,
    baseInstructions: null,
    modelMessages: null,
    supportsReasoningSummaries: booleanCapability(
      model,
      false,
      "supportsReasoningSummaries",
      "supports_reasoning_summaries",
    ),
    defaultReasoningSummary: nullableString(
      capability(model, "defaultReasoningSummary", "default_reasoning_summary"),
    ),
    supportVerbosity: booleanCapability(
      model,
      false,
      "supportsVerbosity",
      "supports_verbosity",
    ),
    defaultVerbosity:
      capability(model, "defaultVerbosity", "default_verbosity") ?? null,
    applyPatchToolType: nullableString(
      capability(model, "applyPatchToolType", "apply_patch_tool_type"),
    ),
    webSearchToolType: nullableString(
      capability(model, "webSearchToolType", "web_search_tool_type"),
    ),
    truncationPolicy: null,
    supportsParallelToolCalls: booleanCapability(
      model,
      false,
      "supportsParallelToolCalls",
      "supports_parallel_tool_calls",
    ),
    supportsImageDetailOriginal: booleanCapability(
      model,
      false,
      "supportsImageDetailOriginal",
      "supports_image_detail_original",
    ),
    contextWindow: model.contextWindow,
    autoCompactTokenLimit: null,
    effectiveContextWindowPercent: null,
    experimentalSupportedTools: [],
    inputModalities: stringList(
      capability(model, "inputModalities", "input_modalities"),
    ),
    minimalClientVersion: null,
    supportsSearchTool: booleanCapability(
      model,
      false,
      "supportsSearchTool",
      "supports_search_tool",
    ),
    availableInPlans: [],
  };
}

export function serializeManagedModelV2ForCodexCache(
  model: ManagedModelV2,
): Record<string, unknown> {
  const reasoningEfforts = stringList(
    capability(model, "reasoningEfforts", "reasoning_efforts"),
  );
  const serviceTiers = stringList(
    capability(model, "serviceTiers", "service_tiers"),
  );
  const inputModalities = stringList(
    capability(model, "inputModalities", "input_modalities"),
  );
  const truncationMode = capability(
    model,
    "truncationMode",
    "truncation_mode",
  );
  const truncationLimit = capability(
    model,
    "truncationLimit",
    "truncation_limit",
  );

  return {
    slug: model.slug,
    display_name: model.displayName || model.slug,
    description: model.description,
    default_reasoning_level: model.defaultReasoningEffort,
    supported_reasoning_levels: reasoningEfforts.map((effort) => ({
      effort,
      description: "",
    })),
    shell_type: "shell_command",
    visibility: model.visibility,
    supported_in_api: model.supportedInApi,
    priority: model.sortOrder,
    additional_speed_tiers: [],
    service_tiers: serviceTiers.map((id) => ({ id, name: id, description: "" })),
    base_instructions: "",
    supports_reasoning_summaries: booleanCapability(
      model,
      false,
      "supportsReasoningSummaries",
      "supports_reasoning_summaries",
    ),
    default_reasoning_summary:
      capability(model, "defaultReasoningSummary", "default_reasoning_summary") ??
      "auto",
    support_verbosity: booleanCapability(
      model,
      false,
      "supportsVerbosity",
      "supports_verbosity",
    ),
    web_search_tool_type:
      capability(model, "webSearchToolType", "web_search_tool_type") ?? "text",
    truncation_policy: {
      mode: typeof truncationMode === "string" ? truncationMode : "tokens",
      limit:
        typeof truncationLimit === "number" && Number.isSafeInteger(truncationLimit)
          ? truncationLimit
          : 10000,
    },
    supports_parallel_tool_calls: booleanCapability(
      model,
      false,
      "supportsParallelToolCalls",
      "supports_parallel_tool_calls",
    ),
    supports_image_detail_original: booleanCapability(
      model,
      false,
      "supportsImageDetailOriginal",
      "supports_image_detail_original",
    ),
    context_window: model.contextWindow,
    effective_context_window_percent: 95,
    experimental_supported_tools: [],
    input_modalities: inputModalities.length > 0 ? inputModalities : ["text", "image"],
    supports_search_tool: booleanCapability(
      model,
      false,
      "supportsSearchTool",
      "supports_search_tool",
    ),
  };
}

export function serializeManagedModelsV2ForCodexCache(
  models: readonly ManagedModelV2[],
): Array<Record<string, unknown>> {
  return [...models]
    .filter(
      (model) =>
        model.enabled && model.supportedInApi && model.visibility === "list",
    )
    .sort(
      (left, right) =>
        left.sortOrder - right.sortOrder || left.slug.localeCompare(right.slug),
    )
    .map(serializeManagedModelV2ForCodexCache);
}

export function buildCodexModelsCachePayloadV2(
  models: readonly ManagedModelV2[],
  userAgent: string,
  options?: { etag?: string | null; fetchedAt?: string },
): Record<string, unknown> {
  const clientVersion = String(userAgent || "")
    .match(/codex_cli_rs\/([^\s]+)/)?.[1]
    ?.trim();
  if (!clientVersion) {
    throw new Error("无法从 userAgent 解析 Codex CLI 版本");
  }
  return {
    fetched_at: options?.fetchedAt || new Date().toISOString(),
    etag: options?.etag ?? null,
    client_version: clientVersion,
    models: serializeManagedModelsV2ForCodexCache(models),
  };
}
