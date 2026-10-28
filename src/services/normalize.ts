import {fileNameOnly} from '../lib/target.ts';
// Normalize only observed data. Missing server metadata stays unknown.
export function normalizeProjects(projects: any[]) {
  for (const project of projects) for (const asset of project.assets || []) {
    asset.fileName = fileNameOnly(asset.name || '');
    asset.hash = asset.sha256 || asset.hash || null;
    asset.relativePath ??= null;
    asset.tags ??= [];
    asset.descriptionStatus ??= 'unknown';
    asset.cutout ??= null;
    asset.savedVersionCount=asset.versions?.length||0;asset.versionCount=asset.savedVersionCount+1;
    asset.processingStatus ??= 'idle';
  }
  return projects;
}
