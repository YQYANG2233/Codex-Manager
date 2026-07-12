import type {
  ManagedModelImportPreviewV2Result,
  ManagedModelImportV2Params,
  ManagedModelListV2Result,
  ManagedModelV2,
  ManagedModelV2Upsert,
} from "@/types/model-v2";

import { invoke, withAddr } from "./transport";

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
