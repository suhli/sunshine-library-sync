<script lang="ts">
	import { Switch as SwitchPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
	import { t } from "$lib/i18n";

	let {
		ref = $bindable(null),
		class: className,
		checked = $bindable(false),
		size = "default",
		...restProps
	}: WithoutChildrenOrChild<SwitchPrimitive.RootProps> & {
		size?: "sm" | "default";
	} = $props();
</script>

<span class="inline-flex min-h-[32px] shrink-0 items-center gap-2 align-middle">
	<SwitchPrimitive.Root
		bind:ref
		bind:checked
		data-slot="switch"
		data-size={size}
		class={cn(
			"peer group/switch relative inline-flex shrink-0 items-center rounded-full border p-[2px] outline-none",
			"data-[size=default]:h-[22px] data-[size=default]:w-[40px] data-[size=sm]:h-[20px] data-[size=sm]:w-[34px]",
			"data-[state=checked]:bg-primary data-[state=checked]:border-primary data-[state=unchecked]:bg-input data-[state=unchecked]:border-muted-foreground",
			"focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive aria-invalid:ring-3",
			"transition-colors duration-150 motion-reduce:transition-none after:absolute after:-inset-x-[4px] after:-inset-y-[6px] data-disabled:cursor-not-allowed data-disabled:opacity-50",
			className
		)}
		{...restProps}
	>
		<SwitchPrimitive.Thumb
			data-slot="switch-thumb"
			class={cn(
				"pointer-events-none block shrink-0 rounded-full bg-white ring-0 dark:data-[state=checked]:bg-primary-foreground",
				"group-data-[size=default]/switch:size-[16px] group-data-[size=sm]/switch:size-[14px]",
				"group-data-[size=default]/switch:data-[state=checked]:translate-x-[18px] group-data-[size=sm]/switch:data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0",
				"rtl:group-data-[size=default]/switch:data-[state=checked]:-translate-x-[18px] rtl:group-data-[size=sm]/switch:data-[state=checked]:-translate-x-[14px]",
				"transition-transform duration-150 motion-reduce:transition-none"
			)}
		/>
	</SwitchPrimitive.Root>
	<span
		aria-hidden="true"
		class={cn("w-[28px] select-none text-xs font-medium", checked ? "text-primary" : "text-muted-foreground", restProps.disabled && "opacity-50")}
	>{$t(checked ? "On" : "Off")}</span>
</span>
