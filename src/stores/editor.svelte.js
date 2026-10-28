import {studio,select} from '../studio.svelte';
import {chooseRatio} from '../lib/ratio.ts';
const state = $state({ versionId: null, coverflowIds:null, ratio: chooseRatio('1:1'), mask:{mode:'erase',size:40},properties: { longEdge: null, mode: 'auto', paddingColor: '#ffffff', backgroundColor: '#ffffff' } });
export const editor = {
  get assetId() { return studio.aid || null; }, open(id) { select(id); },
  get coverflowIds(){return state.coverflowIds;},set coverflowIds(ids){state.coverflowIds=ids===null?null:[...ids];},
  get versionId() { return state.versionId; }, set versionId(value) { state.versionId = value; },
  get tool() { return studio.tool; }, set tool(value) { studio.tool = value; },
  get ratio() { return state.ratio; }, set ratio(value) { state.ratio = value; },
  get properties() { return state.properties; },
  get mask(){return state.mask;},
  get compare() { return studio.compare; }, set compare(value) { studio.compare = value; },
};
