export function planningInstruction(text){
 return `${text}

[Application defaults]
Unless the user explicitly requests another format, output WebP. Dimensions such as 800*800 or 800×800 are final output pixels. 100kb means a maximum of 100000 bytes, not a target of exactly that size. Preserve the requested dimensions; report an unreachable byte limit instead of silently reducing dimensions.
The phrase 产品主图 or product hero means AI-generated product-specific minimalist still-life photography, NOT a generic white-background local export. Use the reference product to choose a suitable restrained scene, natural soft lighting, slightly bright clear tonality and restrained vibrance. Preserve complete product geometry, packaging, material, brand colors, logo and readable text. Ask the image model to position the whole product with safe margins in the target aspect ratio. Do not pre-crop away parts of the product.
Local analysis limitations: the planner receives no image pixels or histogram. Never invent image-measured exposure or saturation numbers or claim histogram-driven auto enhancement. The current local operations do not include an adaptive tone-analysis engine or semantic subject-aware crop. For these specific requests disclose the limitation in warnings; for an explicit local-only adaptive-analysis request use clarification instead of pretending a fixed adjust step is adaptive. Subject-aware framing can be requested from the AI image model; local output pad/crop alone is not semantic framing.
Use concise English plan labels. The user's explicit instructions override defaults.`;
}
export function defaultOutputFormat(text,recipe){
 if(!/\b(jpe?g|png|webp)\b/i.test(text))recipe.output.format='webp';
 return recipe;
}
