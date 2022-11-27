class AActor;
class AController;
class AMatineeActor;
class APawn;
class APlayerController;
class UActorChannel;
class UChildActorComponent;
class UNetDriver;
class UPrimitiveComponent;
struct FAttachedActorInfo;
struct FNetViewer;
struct FNetworkObjectInfo;
class UDataLayer;
enum class EActorUpdateOverlapsMethod : uint8
{
	UseConfigDefault,
	AlwaysUpdate,
	OnlyUpdateMovable,
	NeverUpdate
};
enum class EActorGridPlacement : uint8
{
	Bounds,
	Location,
	AlwaysLoaded,
	None UMETA(Hidden)
};
extern  FUObjectAnnotationSparseBool GSelectedActorAnnotation;
class  AActor : public UObject
{
	AActor();
	AActor(const FObjectInitializer& ObjectInitializer);
	void GetLifetimeReplicatedProps(TArray<FLifetimeProperty>& OutLifetimeProps) const override;
	uint8 bNetTemporary:1;
	uint8 bNetStartup:1;
	uint8 bOnlyRelevantToOwner:1;
	uint8 bAlwaysRelevant:1;    
	virtual void OnRep_ReplicateMovement();
	bool GetTearOff() const
	{
		return bTearOff;
	}
	virtual bool IsInEditingLevelInstance() const
	{
		return bIsInEditingLevelInstance;
	}
	virtual void TearOff();
	uint8 bExchangedRoles:1;
	uint8 bNetLoadOnClient:1;
	uint8 bNetUseOwnerRelevancy:1;
	uint8 bRelevantForNetworkReplays:1;
	uint8 bRelevantForLevelBounds:1;
	uint8 bReplayRewindable:1;
	uint8 bAllowTickBeforeBeginPlay:1;
	uint8 bBlockInput:1;
	uint8 bCollideWhenPlacing:1;
	uint8 bFindCameraComponentWhenViewTarget:1;
	uint8 bGenerateOverlapEventsDuringLevelStreaming:1;
	uint8 bIgnoresOriginShifting:1;
	uint8 bEnableAutoLODGeneration:1;
	uint8 bIsEditorOnlyActor:1;
	uint8 bActorSeamlessTraveled:1;
	virtual bool HasNetOwner() const;
	virtual bool HasLocalNetOwner() const;
	bool GetAutoDestroyWhenFinished() const { return bAutoDestroyWhenFinished; }
	void SetAutoDestroyWhenFinished(bool bVal);
	{
		BeginningPlay,
	};
	static uint32 BeginPlayCallDepth;
	EActorUpdateOverlapsMethod UpdateOverlapsMethodDuringLevelStreaming;
	EActorUpdateOverlapsMethod GetUpdateOverlapsMethodDuringLevelStreaming() const;
	void SetReplicates(bool bInReplicates);
	virtual void SetReplicateMovement(bool bInReplicateMovement);
	void SetAutonomousProxy(const bool bInAutonomousProxy, const bool bAllowForcePropertyCompare=true);
	void CopyRemoteRoleFrom(const AActor* CopyFromActor);
	ENetRole GetLocalRole() const { return Role; }
	ENetRole GetRemoteRole() const;
	float InitialLifeSpan;
	float CustomTimeDilation;
	const struct FRepAttachment& GetAttachmentReplication() const { return AttachmentReplication; }
	virtual void OnRep_AttachmentReplication();
	TEnumAsByte<enum ENetDormancy> NetDormancy;
	virtual bool IsReplicationPausedForConnection(const FNetViewer& ConnectionOwnerNetViewer);
	virtual void OnReplicationPausedChanged(bool bIsReplicationPaused);
	ESpawnActorCollisionHandlingMethod SpawnCollisionHandlingMethod;
	TEnumAsByte<EAutoReceiveInput::Type> AutoReceiveInput;
	int32 InputPriority;
	TObjectPtr<class UInputComponent> InputComponent;
	float NetCullDistanceSquared;   
	int32 NetTag;
	float NetUpdateFrequency;
	float MinNetUpdateFrequency;
	float NetPriority;
	void SetNetDriverName(FName NewNetDriverName);
	FName GetNetDriverName() const { return NetDriverName; }
	virtual bool ReplicateSubobjects(class UActorChannel *Channel, class FOutBunch *Bunch, FReplicationFlags *RepFlags);
	virtual void OnSubobjectCreatedFromReplication(UObject *NewSubobject);
	virtual void OnSubobjectDestroyFromReplication(UObject *Subobject);
	virtual void PreReplication(IRepChangedPropertyTracker & ChangedPropertyTracker);
	virtual void PreReplicationForReplay(IRepChangedPropertyTracker & ChangedPropertyTracker);
	virtual void RewindForReplay();
	void CallPreReplication(UNetDriver* NetDriver);	
	virtual void OnRep_Instigator();
	TArray<TObjectPtr<AActor>> Children;
	void SetHLODLayer(class UHLODLayer* InHLODLayer);
	 bool AllowReceiveTickEventOnDedicatedServer() const { return bAllowReceiveTickEventOnDedicatedServer; }
	 bool IsRunningUserConstructionScript() const { return bRunningUserConstructionScript; }
	TArray< FName > Layers;
	TObjectPtr<AActor> GroupActor;
	float SpriteScale;
	uint64 HiddenEditorViews;
	void SetPackageExternal(bool bExternal, bool bShouldDirty = true);
	FActorOnPackagingModeChanged OnPackagingModeChanged;
	virtual EActorGridPlacement GetDefaultGridPlacement() const;
	virtual EActorGridPlacement GetGridPlacement() const { return GridPlacement; }
	void SetGridPlacement(EActorGridPlacement InGridPLacement) { GridPlacement = InGridPLacement; }
	virtual FName GetRuntimeGrid() const { return RuntimeGrid; }
	void SetRuntimeGrid(FName InRuntimeGrid) { RuntimeGrid = InRuntimeGrid; }
	 const FGuid& GetActorGuid() const { return ActorGuid; }
	virtual bool IsLockLocation() const { return bLockLocation; }
	void SetLockLocation(bool bInLockLocation) { bLockLocation = bInLockLocation; }
	bool IsPackageExternal() const
	{
		return HasAnyFlags(RF_HasExternalPackage);
	}
	uint8 bHiddenEd:1;
	uint8 bIsEditorPreviewActor:1;
	uint8 bHiddenEdLayer:1;
	uint8 bHiddenEdLevel:1;
	uint8 bRunConstructionScriptOnDrag:1;
	TArray<FName> Tags;
	FTakeAnyDamageSignature OnTakeAnyDamage;
	FTakePointDamageSignature OnTakePointDamage;
	FTakeRadialDamageSignature OnTakeRadialDamage;
	FActorBeginOverlapSignature OnActorBeginOverlap;
	FActorEndOverlapSignature OnActorEndOverlap;
	FActorBeginCursorOverSignature OnBeginCursorOver;
	FActorEndCursorOverSignature OnEndCursorOver;
	FActorOnClickedSignature OnClicked;
	FActorOnReleasedSignature OnReleased;
	FActorOnInputTouchBeginSignature OnInputTouchBegin;
	FActorOnInputTouchEndSignature OnInputTouchEnd;
	FActorBeginTouchOverSignature OnInputTouchEnter;
	FActorEndTouchOverSignature OnInputTouchLeave;
	FActorHitSignature OnActorHit;
	virtual void EnableInput(class APlayerController* PlayerController);
	virtual void DisableInput(class APlayerController* PlayerController);
	float GetInputAxisValue(const FName InputAxisName) const;
	float GetInputAxisKeyValue(const FKey InputAxisKey) const;
	FVector GetInputVectorAxisValue(const FKey InputAxisKey) const;
	APawn* GetInstigator() const;
	AController* GetInstigatorController() const;
	bool AddDataLayer(const UDataLayer* DataLayer);
	bool RemoveDataLayer(const UDataLayer* DataLayer);
	bool ContainsDataLayer(const UDataLayer* DataLayer) const;
	virtual bool SupportsDataLayer() const { return true; }
	bool HasDataLayers() const;
	bool HasValidDataLayers() const;
	bool HasAllDataLayers(const TArray<const UDataLayer*>& DataLayers) const;
	bool HasAnyOfDataLayers(const TArray<FName>& DataLayerNames) const;
	TArray<FName> GetDataLayerNames() const;
	TArray<const UDataLayer*> GetDataLayerObjects() const;
	bool IsPropertyChangedAffectingDataLayers(FPropertyChangedEvent& PropertyChangedEvent) const;
	bool IsValidForDataLayer() const;
	void FixupDataLayers();
	const FTransform& GetTransform() const
	{
		return ActorToWorld();
	}
	 const FTransform& ActorToWorld() const
	{
		return (RootComponent ? RootComponent->GetComponentTransform() : FTransform::Identity);
	}
	FVector K2_GetActorLocation() const;
	bool K2_SetActorLocation(FVector NewLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	FRotator K2_GetActorRotation() const;
	FVector GetActorForwardVector() const;
	FVector GetActorUpVector() const;
	FVector GetActorRightVector() const;
	virtual void GetActorBounds(bool bOnlyCollidingComponents, FVector& Origin, FVector& BoxExtent, bool bIncludeFromChildActors = false) const;
	USceneComponent* K2_GetRootComponent() const;
	virtual FVector GetVelocity() const;
	bool SetActorLocation(const FVector& NewLocation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	bool K2_SetActorRotation(FRotator NewRotation, bool bTeleportPhysics);
	bool SetActorRotation(FRotator NewRotation, ETeleportType Teleport = ETeleportType::None);
	bool SetActorRotation(const FQuat& NewRotation, ETeleportType Teleport = ETeleportType::None);
	bool K2_SetActorLocationAndRotation(FVector NewLocation, FRotator NewRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	bool SetActorLocationAndRotation(FVector NewLocation, FRotator NewRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	bool SetActorLocationAndRotation(FVector NewLocation, const FQuat& NewRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void SetActorScale3D(FVector NewScale3D);
	FVector GetActorScale3D() const;
	float GetDistanceTo(const AActor* OtherActor) const;
	float GetSquaredDistanceTo(const AActor* OtherActor) const;
	float GetHorizontalDistanceTo(const AActor* OtherActor) const;
	float GetSquaredHorizontalDistanceTo(const AActor* OtherActor) const;
	float GetVerticalDistanceTo(const AActor* OtherActor) const;
	float GetDotProductTo(const AActor* OtherActor) const;
	float GetHorizontalDotProductTo(const AActor* OtherActor) const;
	void K2_AddActorWorldOffset(FVector DeltaLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorWorldOffset(FVector DeltaLocation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_AddActorWorldRotation(FRotator DeltaRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorWorldRotation(FRotator DeltaRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void AddActorWorldRotation(const FQuat& DeltaRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_AddActorWorldTransform(const FTransform& DeltaTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorWorldTransform(const FTransform& DeltaTransform, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_AddActorWorldTransformKeepScale(const FTransform& DeltaTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorWorldTransformKeepScale(const FTransform& DeltaTransform, bool bSweep = false, FHitResult* OutSweepHitResult = nullptr, ETeleportType Teleport = ETeleportType::None);
	bool K2_SetActorTransform(const FTransform& NewTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	bool SetActorTransform(const FTransform& NewTransform, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_AddActorLocalOffset(FVector DeltaLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorLocalOffset(FVector DeltaLocation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_AddActorLocalRotation(FRotator DeltaRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorLocalRotation(FRotator DeltaRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void AddActorLocalRotation(const FQuat& DeltaRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_AddActorLocalTransform(const FTransform& NewTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void AddActorLocalTransform(const FTransform& NewTransform, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_SetActorRelativeLocation(FVector NewRelativeLocation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void SetActorRelativeLocation(FVector NewRelativeLocation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_SetActorRelativeRotation(FRotator NewRelativeRotation, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void SetActorRelativeRotation(FRotator NewRelativeRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void SetActorRelativeRotation(const FQuat& NewRelativeRotation, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void K2_SetActorRelativeTransform(const FTransform& NewRelativeTransform, bool bSweep, FHitResult& SweepHitResult, bool bTeleport);
	void SetActorRelativeTransform(const FTransform& NewRelativeTransform, bool bSweep=false, FHitResult* OutSweepHitResult=nullptr, ETeleportType Teleport = ETeleportType::None);
	void SetActorRelativeScale3D(FVector NewRelativeScale);
	FVector GetActorRelativeScale3D() const;
	virtual void SetActorHiddenInGame(bool bNewHidden);
	void SetActorEnableCollision(bool bNewActorEnableCollision);
	bool GetActorEnableCollision() const;
	virtual void K2_DestroyActor();
	bool HasAuthority() const;
	UActorComponent* AddComponent(FName TemplateName, bool bManualAttachment, const FTransform& RelativeTransform, const UObject* ComponentTemplateContext, bool bDeferredFinish = false);
	UActorComponent* AddComponentByClass(TSubclassOf<UActorComponent> Class, bool bManualAttachment, const FTransform& RelativeTransform, bool bDeferredFinish);
	void FinishAddComponent(UActorComponent* Component, bool bManualAttachment, const FTransform& RelativeTransform);
	void K2_AttachToComponent(USceneComponent* Parent, FName SocketName, EAttachmentRule LocationRule, EAttachmentRule RotationRule, EAttachmentRule ScaleRule, bool bWeldSimulatedBodies);
	void AttachToComponent(USceneComponent* Parent, const FAttachmentTransformRules& AttachmentRules, FName SocketName = NAME_None);
	void AttachToActor(AActor* ParentActor, const FAttachmentTransformRules& AttachmentRules, FName SocketName = NAME_None);
	void K2_AttachToActor(AActor* ParentActor, FName SocketName, EAttachmentRule LocationRule, EAttachmentRule RotationRule, EAttachmentRule ScaleRule, bool bWeldSimulatedBodies);
	void K2_DetachFromActor(EDetachmentRule LocationRule = EDetachmentRule::KeepRelative, EDetachmentRule RotationRule = EDetachmentRule::KeepRelative, EDetachmentRule ScaleRule = EDetachmentRule::KeepRelative);
	void DetachFromActor(const FDetachmentTransformRules& DetachmentRules);
	void DetachAllSceneComponents(class USceneComponent* InParentComponent, const FDetachmentTransformRules& DetachmentRules);
	bool ActorHasTag(FName Tag) const;
	float GetActorTimeDilation() const;
	float GetActorTimeDilation(const UWorld& ActorWorld) const;
	virtual void AddTickPrerequisiteActor(AActor* PrerequisiteActor);
	virtual void AddTickPrerequisiteComponent(UActorComponent* PrerequisiteComponent);
	virtual void RemoveTickPrerequisiteActor(AActor* PrerequisiteActor);
	virtual void RemoveTickPrerequisiteComponent(UActorComponent* PrerequisiteComponent);
	bool GetTickableWhenPaused();
	void SetTickableWhenPaused(bool bTickableWhenPaused);
	float GetGameTimeSinceCreation() const;
	void DispatchBeginPlay(bool bFromLevelStreaming = false);
	bool IsActorInitialized() const { return bActorInitialized; }
	bool IsActorBeginningPlay() const { return ActorHasBegunPlay == EActorBeginPlayState::BeginningPlay; }
	bool HasActorBegunPlay() const { return ActorHasBegunPlay == EActorBeginPlayState::HasBegunPlay; }
	bool IsActorBeginningPlayFromLevelStreaming() const { return bActorBeginningPlayFromLevelStreaming; }
	bool IsActorBeingDestroyed() const 
	{
		return bActorIsBeingDestroyed;
	}
	void ReceiveAnyDamage(float Damage, const class UDamageType* DamageType, class AController* InstigatedBy, AActor* DamageCauser);
	void ReceiveRadialDamage(float DamageReceived, const class UDamageType* DamageType, FVector Origin, const struct FHitResult& HitInfo, class AController* InstigatedBy, AActor* DamageCauser);
	void ReceivePointDamage(float Damage, const class UDamageType* DamageType, FVector HitLocation, FVector HitNormal, class UPrimitiveComponent* HitComponent, FName BoneName, FVector ShotFromDirection, class AController* InstigatedBy, AActor* DamageCauser, const FHitResult& HitInfo);
	void ReceiveTick(float DeltaSeconds);
	virtual void NotifyActorBeginOverlap(AActor* OtherActor);
	void ReceiveActorBeginOverlap(AActor* OtherActor);
	virtual void NotifyActorEndOverlap(AActor* OtherActor);
	void ReceiveActorEndOverlap(AActor* OtherActor);
	virtual void NotifyActorBeginCursorOver();
	void ReceiveActorBeginCursorOver();
	virtual void NotifyActorEndCursorOver();
	void ReceiveActorEndCursorOver();
	virtual void NotifyActorOnClicked(FKey ButtonPressed = EKeys::LeftMouseButton);
	void ReceiveActorOnClicked(FKey ButtonPressed = EKeys::LeftMouseButton);
	virtual void NotifyActorOnReleased(FKey ButtonReleased = EKeys::LeftMouseButton);
	void ReceiveActorOnReleased(FKey ButtonReleased = EKeys::LeftMouseButton);
	virtual void NotifyActorOnInputTouchBegin(const ETouchIndex::Type FingerIndex);
	void ReceiveActorOnInputTouchBegin(const ETouchIndex::Type FingerIndex);
	virtual void NotifyActorOnInputTouchEnd(const ETouchIndex::Type FingerIndex);
	void ReceiveActorOnInputTouchEnd(const ETouchIndex::Type FingerIndex);
	virtual void NotifyActorOnInputTouchEnter(const ETouchIndex::Type FingerIndex);
	void ReceiveActorOnInputTouchEnter(const ETouchIndex::Type FingerIndex);
	virtual void NotifyActorOnInputTouchLeave(const ETouchIndex::Type FingerIndex);
	void ReceiveActorOnInputTouchLeave(const ETouchIndex::Type FingerIndex);
	void GetOverlappingActors(TArray<AActor*>& OverlappingActors, TSubclassOf<AActor> ClassFilter=nullptr) const;
	void GetOverlappingActors(TSet<AActor*>& OverlappingActors, TSubclassOf<AActor> ClassFilter=nullptr) const;
	void GetOverlappingComponents(TArray<UPrimitiveComponent*>& OverlappingComponents) const;
	void GetOverlappingComponents(TSet<UPrimitiveComponent*>& OverlappingComponents) const;
	virtual void NotifyHit(class UPrimitiveComponent* MyComp, AActor* Other, class UPrimitiveComponent* OtherComp, bool bSelfMoved, FVector HitLocation, FVector HitNormal, FVector NormalImpulse, const FHitResult& Hit);
	void ReceiveHit(class UPrimitiveComponent* MyComp, AActor* Other, class UPrimitiveComponent* OtherComp, bool bSelfMoved, FVector HitLocation, FVector HitNormal, FVector NormalImpulse, const FHitResult& Hit);
	virtual void SetLifeSpan( float InLifespan );
	virtual float GetLifeSpan() const;
	void UserConstructionScript();
	bool Destroy(bool bNetForce = false, bool bShouldModifyLevel = true );
	void ReceiveDestroyed();
	FActorDestroyedSignature OnDestroyed;
	FActorEndPlaySignature OnEndPlay;
	virtual bool CheckDefaultSubobjectsInternal() const override;
	virtual void PostInitProperties() override;
	virtual void ProcessEvent( UFunction* Function, void* Parameters ) override;
	virtual int32 GetFunctionCallspace( UFunction* Function, FFrame* Stack ) override;
	virtual bool CallRemoteFunction( UFunction* Function, void* Parameters, FOutParmRec* OutParms, FFrame* Stack ) override;
	virtual void Serialize(FArchive& Ar) override;
	virtual void PostLoad() override;
	virtual void PostLoadSubobjects( FObjectInstancingGraph* OuterInstanceGraph ) override;
	virtual void BeginDestroy() override;
	virtual bool IsReadyForFinishDestroy() override;
	virtual bool Rename( const TCHAR* NewName=nullptr, UObject* NewOuter=nullptr, ERenameFlags Flags=REN_None ) override;
	virtual void PostRename( UObject* OldOuter, const FName OldName ) override;
	virtual bool CanBeInCluster() const override;
	static void AddReferencedObjects(UObject* InThis, FReferenceCollector& Collector);
	virtual bool IsEditorOnly() const override;
	virtual bool IsAsset() const override;
	virtual bool PreSaveRoot(const TCHAR* InFilename) override;
	virtual void PostSaveRoot(bool bCleanupIsRequired) override;
	virtual void PreSave(const ITargetPlatform* TargetPlatform) override;
	bool IsMainPackageActor() const;
	static AActor* FindActorInPackage(UPackage* InPackage);
	virtual bool Modify(bool bAlwaysMarkDirty = true) override;
	virtual void GetAssetRegistryTags(TArray<FAssetRegistryTag>& OutTags) const override;
	virtual void GetExternalActorExtendedAssetRegistryTags(TArray<FAssetRegistryTag>& OutTags) const override;
	virtual bool NeedsLoadForTargetPlatform(const ITargetPlatform* TargetPlatform) const;
	virtual void PreEditChange(FProperty* PropertyThatWillChange) override;
	virtual bool CanEditChange(const FProperty* InProperty) const;
	virtual void PostEditChangeProperty(FPropertyChangedEvent& PropertyChangedEvent) override;
	virtual void PreEditUndo() override;
	virtual void PostEditUndo() override;
	virtual void PostEditImport() override;
	virtual void PostTransacted(const FTransactionObjectEvent& TransactionEvent) override;
	virtual bool IsSelectedInEditor() const override;
	virtual bool IsUserManaged() const { return true; }
	virtual bool CanDeleteSelectedActor(FText& OutReason) const;
	virtual bool SupportsExternalPackaging() const;