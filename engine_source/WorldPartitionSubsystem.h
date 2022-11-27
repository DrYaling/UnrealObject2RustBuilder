// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "Subsystems/WorldSubsystem.h"
#include "Templates/SubclassOf.h"
#include "Tickable.h"
#include "WorldPartition/WorldPartitionStreamingSource.h"
#include "WorldPartitionSubsystem.generated.h"

class UWorldPartition;
class UWorldPartitionEditorCell;
class FWorldPartitionActorDesc;

enum class EWorldPartitionRuntimeCellState : uint8;

/**
 * UWorldPartitionSubsystem
 */

UCLASS()
class ENGINE_API UWorldPartitionSubsystem : public UWorldSubsystem, public FTickableGameObject
{
	GENERATED_BODY()

public:
	UWorldPartitionSubsystem();

	//~ Begin USubsystem Interface.
	virtual bool ShouldCreateSubsystem(UObject* Outer) const override;
	virtual void Deinitialize() override;
	//~ End USubsystem Interface.

	//~ Begin UWorldSubsystem Interface.
	virtual void PostInitialize() override;
	virtual void UpdateStreamingState() override;
	//~ End UWorldSubsystem Interface.

	//~ Begin FTickableGameObject
	virtual void Tick(float DeltaSeconds) override;
	virtual bool IsTickableInEditor() const override;
	virtual UWorld* GetTickableGameObjectWorld() const override;
	virtual ETickableTickType GetTickableTickType() const override;
	virtual TStatId GetStatId() const override;
	//~End FTickableGameObject

	DECLARE_MULTICAST_DELEGATE_OneParam(FWorldPartitionRegisterDelegate, UWorldPartition*);
	FWorldPartitionRegisterDelegate OnWorldPartitionRegistered;
	FWorldPartitionRegisterDelegate OnWorldPartitionUnregistered;

	UFUNCTION(BlueprintCallable, Category = Streaming)
	bool IsStreamingCompleted(EWorldPartitionRuntimeCellState QueryState, const TArray<FWorldPartitionStreamingQuerySource>& QuerySources, bool bExactState) const;

#if WITH_EDITOR
	void ForEachIntersectingActorDesc(const FBox& Box, TSubclassOf<AActor> ActorClass, TFunctionRef<bool(const FWorldPartitionActorDesc*)> Predicate) const;
	void ForEachActorDesc(TSubclassOf<AActor> ActorClass, TFunctionRef<bool(const FWorldPartitionActorDesc*)> Predicate) const;
#endif
	void ToggleDrawRuntimeHash2D();

private:
	UWorldPartition* GetMainWorldPartition();
	const UWorldPartition* GetMainWorldPartition() const;
	void RegisterWorldPartition(UWorldPartition* WorldPartition);
	void UnregisterWorldPartition(UWorldPartition* WorldPartition);
	void Draw(class UCanvas* Canvas, class APlayerController* PC);
	friend class UWorldPartition;

	UPROPERTY()
	TArray<TObjectPtr<UWorldPartition>> RegisteredWorldPartitions;

	FDelegateHandle	DrawHandle;
};
